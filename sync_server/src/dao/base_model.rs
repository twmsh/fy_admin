use chrono::{DateTime, Local};
use mysql_codegen::MysqlEntity;
use serde::{Deserialize, Serialize};
use sqlx::Arguments;

use fy_base::util::mysql_util;

/* 小盒子 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_box"]
pub struct BaseBox {
    /* id */
    #[pk]
    pub id: i64,

    /* 盒子名称 */
    pub name: Option<String>,

    /* 硬件编号 */
    pub hw_id: String,

    /* 盒子编号;应用层面分配的 */
    pub device_id: String,

    /* 同步状态开关;0:同步关闭 1:同步开启 */
    pub sync_flag: i16,

    /* 是否保存db;0: 不需同步db 1:需要同步db */
    pub has_db: i16,

    /* 是否有摄像头;0: 不需要同步摄像头 1:需要同步摄像头 */
    pub has_camera: i16,

    /* 最新上线时间 */
    pub latest_online: Option<DateTime<Local>>,

    /* 录入时间 */
    pub create_time: DateTime<Local>,

    /* 修改时间 */
    pub modify_time: DateTime<Local>,
}
/* 小盒子日志 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_box_log"]
pub struct BaseBoxLog {
    /* id */
    #[pk]
    pub id: i64,

    /* 小盒子硬件编号 */
    pub box_hwid: String,

    /* 小盒子ip列表;逗号分隔 */
    pub box_ips: String,

    /* 日志类别 */
    pub log_type: String,

    /* 日志级别;0:debug, 1: info, 2: warn, 3: error */
    pub log_level: i16,

    /* 日志内容 */
    pub log_payload: String,

    /* 创建时间 */
    pub create_time: DateTime<Local>,
}
/* 摄像头 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_camera"]
pub struct BaseCamera {
    /* id */
    #[pk]
    pub id: i64,

    /* 摄像头名称 */
    pub name: Option<String>,

    /* uuid */
    pub uuid: String,

    /* 小盒子编号 */
    pub box_deviceid: String,

    /* 摄像头采集类型 */
    pub c_type: i16,

    /* 采集地址 */
    pub url: String,

    /* 摄像头配置 */
    pub config: String,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间 */
    pub modify_time: DateTime<Local>,
}
/* 摄像头删除表 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_camera_del"]
pub struct BaseCameraDel {
    /* id */
    #[pk]
    pub id: i64,

    /* 原来表中的id */
    pub origin_id: i32,

    /* 摄像头名称 */
    pub name: Option<String>,

    /* uuid */
    pub uuid: String,

    /* 小盒子编号 */
    pub box_deviceid: String,

    /* 摄像头采集类型 */
    pub c_type: i16,

    /* 采集地址 */
    pub url: String,

    /* 摄像头配置 */
    pub config: String,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间;删除时间 */
    pub modify_time: DateTime<Local>,
}
/* 特征库 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_db"]
pub struct BaseDb {
    /* id */
    #[pk]
    pub id: i64,

    /* uuid */
    pub uuid: String,

    /* 容量 */
    pub capacity: i32,

    /* 使用量 */
    pub uses: i32,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间 */
    pub modify_time: DateTime<Local>,
}
/* 特征库删除表 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_db_del"]
pub struct BaseDbDel {
    /* id */
    #[pk]
    pub id: i64,

    /* 原来表中的id */
    pub origin_id: i32,

    /* uuid */
    pub uuid: String,

    /* 容量 */
    pub capacity: i32,

    /* 使用量 */
    pub uses: i32,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间;删除时间 */
    pub modify_time: DateTime<Local>,
}
/* 特征值 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_fea"]
pub struct BaseFea {
    /* id */
    #[pk]
    pub id: i64,

    /* uuid */
    pub uuid: String,

    /* db uuid */
    pub db_uuid: String,

    /* 特征值(聚合) */
    pub feature: Option<String>,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间 */
    pub modify_time: DateTime<Local>,
}
/* 特征值删除表 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_fea_del"]
pub struct BaseFeaDel {
    /* id */
    #[pk]
    pub id: i64,

    /* 原来表中的id */
    pub origin_id: i32,

    /* uuid */
    pub uuid: String,

    /* db uuid */
    pub db_uuid: String,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间;删除时间 */
    pub modify_time: DateTime<Local>,
}
/* 特征值映射表 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "base_fea_map"]
pub struct BaseFeaMap {
    /* id */
    #[pk]
    pub id: i64,

    /* uuid */
    pub uuid: String,

    /* 图片编号;调试用，用来对应person的人脸编号 */
    pub face_id: String,

    /* 特征值 */
    pub feature: String,

    /* 图片质量 */
    pub quality: f32,

    /* 创建时间 */
    pub create_time: DateTime<Local>,

    /* 更新时间 */
    pub modify_time: DateTime<Local>,
}
/* 车辆抓拍记录 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "cartrack"]
pub struct Cartrack {
    /* id */
    #[pk]
    pub id: i64,

    /* uuid */
    pub uuid: String,

    /* 摄像头uuid */
    pub camera_uuid: String,

    /* 图片ids;index:quality,index:quality */
    pub img_ids: String,

    /* 车牌是否识别出;0：否，1：是 */
    pub plate_judged: i16,

    /* 车型是否识别出来;0：否，1：是 */
    pub vehicle_judged: i16,

    /* 运动方向;0 未知；1 向上；2 向下 */
    pub move_direct: i16,

    /* 车辆方位;uuid:score,uuid:score */
    pub car_direct: Option<String>,

    /* 车牌号 */
    pub plate_content: Option<String>,

    /* 车牌置信度 */
    pub plate_confidence: Option<f32>,

    /* 车牌类型 */
    pub plate_type: Option<String>,

    /* 车身颜色 */
    pub car_color: Option<String>,

    /* 品牌 */
    pub car_brand: Option<String>,

    /* 车系 */
    pub car_top_series: Option<String>,

    /* 车款 */
    pub car_series: Option<String>,

    /* 车粗分类别 */
    pub car_top_type: Option<String>,

    /* 车类别 */
    pub car_mid_type: Option<String>,

    /* 抓拍时间 */
    pub capture_time: DateTime<Local>,

    /* 创建时间 */
    pub create_time: DateTime<Local>,
}
/* 人脸抓拍记录 */
#[derive(sqlx::FromRow, MysqlEntity, Serialize, Deserialize, Debug, Clone)]
#[table = "facetrack"]
pub struct Facetrack {
    /* id */
    #[pk]
    pub id: i64,

    /* uuid */
    pub uuid: String,

    /* 摄像头uuid */
    pub camera_uuid: String,

    /* 图片ids;index:quality,index:quality */
    pub img_ids: String,

    /* 特征值ids;index:quality,index:quality */
    pub feature_ids: String,

    /* 性别 */
    pub gender: i16,

    /* 年龄 */
    pub age: i16,

    /* 眼镜 */
    pub glasses: i16,

    /* TOP-N匹配到的人列表;uuid:score,uuid:score */
    pub most_persons: Option<String>,

    /* 抓拍时间 */
    pub capture_time: DateTime<Local>,

    /* 创建时间 */
    pub create_time: DateTime<Local>,
}
