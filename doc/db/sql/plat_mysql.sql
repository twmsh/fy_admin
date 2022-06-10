DROP TABLE IF EXISTS base_box;
CREATE TABLE base_box(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    name VARCHAR(50)    COMMENT '盒子名称' ,
    hw_id VARCHAR(50) NOT NULL   COMMENT '硬件编号' ,
    sync_flag SMALLINT NOT NULL   COMMENT '同步状态开关;0:同步关闭 1:同步开启' ,
    has_db SMALLINT NOT NULL   COMMENT '是否保存db;0: 不需同步db 1:需要同步db' ,
    has_camera SMALLINT NOT NULL   COMMENT '是否有摄像头;0: 不需要同步摄像头 1:需要同步摄像头' ,
    latest_online DATETIME(3)    COMMENT '最新上线时间' ,
    create_time DATETIME(3) NOT NULL   COMMENT '录入时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '修改时间' ,
    PRIMARY KEY (id)
)  COMMENT = '小盒子';


CREATE UNIQUE INDEX idx_box_hw_id ON base_box(hw_id);

DROP TABLE IF EXISTS base_camera;
CREATE TABLE base_camera(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    name VARCHAR(50)    COMMENT '摄像头名称' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    box_hwid VARCHAR(50) NOT NULL   COMMENT '小盒子硬件编号' ,
    url VARCHAR(255) NOT NULL   COMMENT '采集地址' ,
    config TEXT NOT NULL   COMMENT '摄像头配置' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '摄像头';


CREATE UNIQUE INDEX idx_camera_uuid ON base_camera(uuid);
CREATE INDEX idx_camera_box_hwid ON base_camera(box_hwid);
CREATE INDEX idx_camera_modify ON base_camera(modify_time);

DROP TABLE IF EXISTS base_db;
CREATE TABLE base_db(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    capacity INT NOT NULL   COMMENT '容量' ,
    uses INT NOT NULL   COMMENT '使用量' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征库';


CREATE UNIQUE INDEX idx_db_uuid ON base_db(uuid);
CREATE INDEX idx_db_modify ON base_db(modify_time);

DROP TABLE IF EXISTS base_fea;
CREATE TABLE base_fea(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    db_uuid VARCHAR(50) NOT NULL   COMMENT 'db uuid' ,
    feature TEXT    COMMENT '特征值(聚合)' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征值';


CREATE UNIQUE INDEX idx_fea_uuid ON base_fea(uuid);
CREATE INDEX idx_fea_dbuuid ON base_fea(db_uuid);
CREATE INDEX idx_fea_modify ON base_fea(modify_time);

DROP TABLE IF EXISTS base_box_log;
CREATE TABLE base_box_log(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    box_hwid VARCHAR(50) NOT NULL   COMMENT '小盒子硬件编号' ,
    log_type VARCHAR(50) NOT NULL   COMMENT '日志类别' ,
    log_level SMALLINT NOT NULL   COMMENT '日志级别;0:debug, 1: info, 2: warn, 3: error' ,
    log_payload TEXT    COMMENT '日志内容' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    PRIMARY KEY (id)
)  COMMENT = '小盒子日志';


CREATE INDEX idx_boxlog_hwid ON base_box_log(box_hwid);
CREATE INDEX idx_boxlog_logtype ON base_box_log(log_type);
CREATE INDEX idx_boxlog_create ON base_box_log(create_time);

DROP TABLE IF EXISTS facetrack;
CREATE TABLE facetrack(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
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

DROP TABLE IF EXISTS base_db_del;
CREATE TABLE base_db_del(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    origin_id INT NOT NULL   COMMENT '原来表中的id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    capacity INT NOT NULL   COMMENT '容量' ,
    uses INT NOT NULL   COMMENT '使用量' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间;删除时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征库删除表';


CREATE INDEX idx_db_del_uuid ON base_db_del(uuid);
CREATE INDEX idx_db_del_modify ON base_db_del(modify_time);

DROP TABLE IF EXISTS base_fea_del;
CREATE TABLE base_fea_del(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    origin_id INT NOT NULL   COMMENT '原来表中的id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    db_uuid VARCHAR(50) NOT NULL   COMMENT 'db uuid' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间;删除时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征值删除表';


CREATE INDEX idx_fea_del_uuid ON base_fea_del(uuid);
CREATE INDEX idx_fea_del_dbuuid ON base_fea_del(db_uuid);
CREATE INDEX idx_fea_del_modify ON base_fea_del(modify_time);

DROP TABLE IF EXISTS base_camera_del;
CREATE TABLE base_camera_del(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    origin_id INT NOT NULL   COMMENT '原来表中的id' ,
    name VARCHAR(50)    COMMENT '摄像头名称' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    box_hwid VARCHAR(50) NOT NULL   COMMENT '小盒子硬件编号' ,
    url VARCHAR(255) NOT NULL   COMMENT '采集地址' ,
    config TEXT NOT NULL   COMMENT '摄像头配置' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间;删除时间' ,
    PRIMARY KEY (id)
)  COMMENT = '摄像头删除表';


CREATE INDEX idx_camera_del_uuid ON base_camera_del(uuid);
CREATE INDEX idx_camera_del_box_hwid ON base_camera_del(box_hwid);
CREATE INDEX idx_camera_del_modify ON base_camera_del(modify_time);

DROP TABLE IF EXISTS base_fea_map;
CREATE TABLE base_fea_map(
    id BIGINT NOT NULL AUTO_INCREMENT  COMMENT 'id' ,
    uuid VARCHAR(50) NOT NULL   COMMENT 'uuid' ,
    face_id VARCHAR(10) NOT NULL   COMMENT '图片编号;调试用，用来对应person的人脸编号' ,
    feature TEXT NOT NULL   COMMENT '特征值' ,
    quality DECIMAL(6,3) NOT NULL   COMMENT '图片质量' ,
    create_time DATETIME(3) NOT NULL   COMMENT '创建时间' ,
    modify_time DATETIME(3) NOT NULL   COMMENT '更新时间' ,
    PRIMARY KEY (id)
)  COMMENT = '特征值映射表';


CREATE INDEX idx_fea_map_uuid ON base_fea_map(uuid);

