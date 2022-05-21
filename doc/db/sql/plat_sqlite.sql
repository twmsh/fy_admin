DROP TABLE IF EXISTS box;
CREATE TABLE box
    -- 小盒子
(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- id
    name VARCHAR(50), -- 盒子名称
    hw_id VARCHAR(50) NOT NULL, -- 硬件编号
    has_db SMALLINT NOT NULL, -- 是否保存db
    latest_online DATETIME(3), -- 最新上线时间
    create_time DATETIME(3) NOT NULL, -- 录入时间
    modify_time DATETIME(3) NOT NULL -- 修改时间
);


CREATE UNIQUE INDEX idx_box_hw_id ON box(hw_id);

DROP TABLE IF EXISTS camera;
CREATE TABLE camera
    -- 摄像头
(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- id
    name VARCHAR(50), -- 摄像头名称
    uuid VARCHAR(50) NOT NULL, -- uuid
    box_hwid VARCHAR(50) NOT NULL, -- 小盒子硬件编号
    url VARCHAR(255) NOT NULL, -- 采集地址
    config TEXT NOT NULL, -- 摄像头配置
    create_time DATETIME(3) NOT NULL, -- 创建时间
    modify_time DATETIME(3) NOT NULL -- 更新时间
);


CREATE UNIQUE INDEX idx_camera_uuid ON camera(uuid);
CREATE INDEX idx_camera_box_hwid ON camera(box_hwid);
CREATE INDEX idx_camera_modify ON camera(modify_time);

DROP TABLE IF EXISTS db;
CREATE TABLE db
    -- 特征库
(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- id
    uuid VARCHAR(50) NOT NULL, -- uuid
    capacity INTEGER NOT NULL, -- 容量
    uses INTEGER NOT NULL, -- 使用量
    create_time DATETIME(3) NOT NULL, -- 创建时间
    modify_time DATETIME(3) NOT NULL -- 更新时间
);


CREATE UNIQUE INDEX idx_db_uuid ON db(uuid);
CREATE INDEX idx_db_modify ON db(modify_time);

DROP TABLE IF EXISTS fea;
CREATE TABLE fea
    -- 特征值
(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- id
    uuid VARCHAR(50) NOT NULL, -- uuid
    db_uuid VARCHAR(50) NOT NULL, -- db uuid
    feature TEXT NOT NULL, -- 特征值
    create_time DATETIME(3) NOT NULL, -- 创建时间
    modify_time DATETIME(3) NOT NULL -- 更新时间
);


CREATE UNIQUE INDEX idx_fea_uuid ON fea(uuid);
CREATE INDEX idx_fea_dbuuid ON fea(db_uuid);
CREATE INDEX idx_fea_modify ON fea(modify_time);

DROP TABLE IF EXISTS box_log;
CREATE TABLE box_log
    -- 小盒子日志
(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- id
    box_hwid VARCHAR(50) NOT NULL, -- 小盒子硬件编号
    log_type VARCHAR(50) NOT NULL, -- 日志类别
    log_payload TEXT, -- 日志内容
    create_time DATETIME(3) NOT NULL -- 创建时间
);


CREATE INDEX idx_boxlog_hwid ON box_log(box_hwid);
CREATE INDEX idx_boxlog_logtype ON box_log(log_type);
CREATE INDEX idx_boxlog_create ON box_log(create_time);

DROP TABLE IF EXISTS facetrack;
CREATE TABLE facetrack
    -- 人脸抓拍记录
(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- id
    uuid VARCHAR(50) NOT NULL, -- uuid
    camera_uuid VARCHAR(50) NOT NULL, -- 摄像头uuid
    img_ids VARCHAR(400) NOT NULL, -- 图片ids;index:quality,index:quality
    feature_ids VARCHAR(400) NOT NULL, -- 特征值ids;index:quality,index:quality
    gender SMALLINT NOT NULL DEFAULT 0, -- 性别
    age SMALLINT NOT NULL DEFAULT 0, -- 年龄
    glasses SMALLINT NOT NULL DEFAULT 0, -- 眼镜
    most_persons VARCHAR(400), -- TOP-N匹配到的人列表;uuid:score,uuid:score
    capture_time DATETIME NOT NULL, -- 抓拍时间
    create_time DATETIME NOT NULL -- 创建时间
);


CREATE UNIQUE INDEX idx_facetrack_uuid ON facetrack(uuid);
CREATE INDEX idx_facetrack_cameraid ON facetrack(camera_uuid);
CREATE INDEX idx_facetrack_capturetime ON facetrack(capture_time);

