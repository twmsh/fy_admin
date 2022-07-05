# fy_admin
rust开发的产品平台，包括web服务端和一些微服务，使用到的一些技术栈和库： 
* clap, log4rs, tracing
* tokio, axum, tower, reqwest
* sqlx (mysql)
* lapin (rabbitmq)
* rust-s3 (minio)
* ..

## 模块
+ box_agent 小盒子(arm64)运行的采集/比对/上传程序

+ doc 文档说明

+ fy_base 基础库

+ mysql_codegen 属性宏库，用来自动生成一些数据库操作代码。
用自有的代码生成工具，读取数据库信息，对每张单生成一个带有属性宏标注的rust struct代码

+ sync_client 运行在小盒子(arm64)上，与中心同步服务器进行同步

+ sync_server 运行在x86服务器上，提供同步服务

+ track_warehouse 运行在x86服务器上，将小盒子采集的数据保存,（文件保存到minio,记录保存到mysql）,然后将采集记录提交到rabbitmq，供其他模块调用。
