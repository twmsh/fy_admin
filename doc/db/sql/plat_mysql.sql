DROP TABLE IF EXISTS box;
CREATE TABLE box(
    id INT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    name VARCHAR(50)    COMMENT '盒子名称' ,
    hw_id VARCHAR(50) NOT NULL   COMMENT '硬件编号' ,
    has_db SMALLINT NOT NULL   COMMENT '是否保存db' ,
    latest_online DATETIME(3)    COMMENT '最新上线时间' ,
    create_time DATETIME(3) NOT NULL   COMMENT '录入时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '修改时间' ,
    PRIMARY KEY (id)
)  COMMENT = '小盒子';


CREATE UNIQUE INDEX idx_box_hw_id ON box(hw_id);

DROP TABLE IF EXISTS camera;
CREATE TABLE camera(
    id INT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    name VARCHAR(50)    COMMENT '摄像头名称' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    box_hwid VARCHAR(50) NOT NULL   COMMENT '小盒子硬件编号' ,
    url VARCHAR(255) NOT NULL   COMMENT '采集地址' ,
    config TEXT NOT NULL   COMMENT '摄像头配置' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '摄像头';


CREATE UNIQUE INDEX idx_camera_uuid ON camera(uuid);
CREATE INDEX idx_camera_box_hwid ON camera(box_hwid);
CREATE INDEX idx_camera_modify ON camera(modify_time);

DROP TABLE IF EXISTS db;
CREATE TABLE db(
    id INT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    capacity INT NOT NULL   COMMENT '容量' ,
    uses INT NOT NULL   COMMENT '使用量' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征库';


CREATE UNIQUE INDEX idx_db_uuid ON db(uuid);
CREATE INDEX idx_db_modify ON db(modify_time);

DROP TABLE IF EXISTS fea;
CREATE TABLE fea(
    id INT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    db_uuid VARCHAR(50) NOT NULL   COMMENT 'db uuid' ,
    feature TEXT NOT NULL   COMMENT '特征值' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征值';


CREATE UNIQUE INDEX idx_fea_uuid ON fea(uuid);
CREATE INDEX idx_fea_dbuuid ON fea(db_uuid);
CREATE INDEX idx_fea_modify ON fea(modify_time);

DROP TABLE IF EXISTS box_log;
CREATE TABLE box_log(
    id INT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    box_hwid VARCHAR(50) NOT NULL   COMMENT '小盒子硬件编号' ,
    log_type VARCHAR(50) NOT NULL   COMMENT '日志类别' ,
    log_payload TEXT    COMMENT '日志内容' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    PRIMARY KEY (id)
)  COMMENT = '小盒子日志';


CREATE INDEX idx_boxlog_hwid ON box_log(box_hwid);
CREATE INDEX idx_boxlog_logtype ON box_log(log_type);
CREATE INDEX idx_boxlog_create ON box_log(create_time);

DROP TABLE IF EXISTS facetrack;
CREATE TABLE facetrack(
    id INT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    camera_uuid VARCHAR(50) NOT NULL   COMMENT '摄像头uuid' ,
    img_ids VARCHAR(400) NOT NULL   COMMENT '图片ids;index:quality,index:quality' ,
    feature_ids VARCHAR(400) NOT NULL   COMMENT '特征值ids;index:quality,index:quality' ,
    gender SMALLINT NOT NULL  DEFAULT 0 COMMENT '性别' ,
    age SMALLINT NOT NULL  DEFAULT 0 COMMENT '年龄' ,
    glasses SMALLINT NOT NULL  DEFAULT 0 COMMENT '眼镜' ,
    most_persons VARCHAR(400)    COMMENT 'TOP-N匹配到的人列表;uuid:score,uuid:score' ,
    capture_time DATETIME NOT NULL   COMMENT '抓拍时间' ,
    create_time DATETIME NOT NULL   COMMENT '创建时间' ,
    PRIMARY KEY (id)
)  COMMENT = '人脸抓拍记录';


CREATE UNIQUE INDEX idx_facetrack_uuid ON facetrack(uuid);
CREATE INDEX idx_facetrack_cameraid ON facetrack(camera_uuid);
CREATE INDEX idx_facetrack_capturetime ON facetrack(capture_time);

