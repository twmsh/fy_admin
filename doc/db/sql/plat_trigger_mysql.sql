-- trigger for base_camera
DROP TRIGGER IF EXISTS trg_base_camera_del;
CREATE TRIGGER trg_base_camera_del
AFTER DELETE
   ON base_camera FOR EACH ROW

BEGIN

  INSERT INTO base_camera_del(origin_id,name,uuid,box_hwid,url,config,create_time,modify_time)
  VALUES(OLD.id, OLD.name,OLD.uuid,OLD.box_hwid,OLD.url,OLD.config,OLD.create_time,now(3));

END;


-- trigger for base_db
DROP TRIGGER IF EXISTS trg_base_db_del;
CREATE TRIGGER trg_base_db_del
AFTER DELETE
   ON base_db FOR EACH ROW

BEGIN

  INSERT INTO base_db_del(origin_id,uuid,capacity,uses,create_time,modify_time)
  VALUES(OLD.id, OLD.uuid,OLD.capacity,OLD.uses,OLD.create_time,now(3));

END;


-- trigger for base_fea
DROP TRIGGER IF EXISTS trg_base_fea_del;
CREATE TRIGGER trg_base_fea_del
AFTER DELETE
   ON base_fea FOR EACH ROW

BEGIN

  INSERT INTO base_fea_del(origin_id,uuid,db_uuid,create_time,modify_time)
  VALUES(OLD.id, OLD.uuid,OLD.db_uuid,OLD.create_time,now(3));

END;