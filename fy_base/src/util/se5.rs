use ini::Ini;
use std::io;

/*
/factory/OEMconfig.ini
[BASE]
SN = HQDZKM6BJJIBF0089
MAC0 = e0 a5 09 00 36 98
MAC1 = e0 a5 09 00 36 9e
PRODUCT_TYPE = 0x03
AGING_FLAG = 0x01
DDR_TYPE = 0x31
BOARD_TYPE = V12
BOM = V12
MODULE_TYPE = SE50221
EX_MODULE_TYPE = SE50221
PRODUCT = SE5
VENDER = BITMAIN
ALGORITHM = BITMAIN
DEVICE_SN = HQDZKE6BJJIBE0083
DATE_PRODUCTION =
PASSWORD_SSH = linaro
USERNAME = admin
PASSWORD = admin

[CONFIG_PARSER]
MD5 = 78b5b3b9f46eab2117d6b16f41224797
enable_hash_verification = true

----------------
/sys/bus/i2c/devices/1-0017/information
{
        "model": "SE5",
        "chip": "BM1684",
        "mcu": "STM32",
        "product sn": "HQDZKM6BJJIBF0089",
        "board type": "0x03",
        "mcu version": "0x2C",
        "pcb version": "0x11",
        "reset count": 0
}
*/

// 获取 小盒子 唯一串号
// 方法1：读取 /factory/OEMconfig.ini 文件中的 DEVICE_SN, 这个值和SE5设备底部的贴纸上的SN一直。 (还有一个SN参数,可能是芯片的SN）
// 方法2：读取 sys/bus/i2c/devices/1-0017/information 中的 "product sn", 这个值和方法一种的SN相同

pub fn get_device_sn() -> io::Result<String> {
    let path = "/factory/OEMconfig.ini";

    let conf = match Ini::load_from_file(path) {
        Ok(v) => v,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    let section = match conf.section(Some("BASE")) {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "BASE section not found",
            ));
        }
        Some(v) => v,
    };

    let value = match section.get("DEVICE_SN") {
        None => {
            return Err(io::Error::new(io::ErrorKind::Other, "DEVICE_SN not found"));
        }
        Some(v) => v,
    };

    Ok(value.to_string())
}

pub fn get_chip_sn() -> std::io::Result<String> {
    let path = "/factory/OEMconfig.ini";

    let conf = match Ini::load_from_file(path) {
        Ok(v) => v,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    let section = match conf.section(Some("BASE")) {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "BASE section not found",
            ));
        }
        Some(v) => v,
    };

    let value = match section.get("SN") {
        None => {
            return Err(io::Error::new(io::ErrorKind::Other, "SN not found"));
        }
        Some(v) => v,
    };

    Ok(value.to_string())
}
