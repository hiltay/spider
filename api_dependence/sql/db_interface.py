import os
import sys
from db import models
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, scoped_session
from tools import utils

class SQLEngine(object):
    engine = None

    def __new__(cls):
        if cls.engine is None:
            cls.engine = cls.__get_sql_engine()
            # 创建表
            try:
                models.Model.metadata.create_all(cls.engine)
            except:
                pass
        return cls.engine

    @staticmethod
    def __get_sql_engine():
        settings = utils.get_user_settings()
        base_path = utils.get_base_path()
        db_path = os.path.join(base_path, 'data.db')
        if os.environ.get("DEBUG"):
            if settings["DATABASE"] == "sqlite":
                if sys.platform == "win32":
                    conn = rf"sqlite:///{db_path}?check_same_thread=False"
                else:
                    conn = f"sqlite:////{db_path}?check_same_thread=False"
                # conn = "sqlite:///" + BASE_DIR + "/data.db" + "?check_same_thread=False"
            elif settings["DATABASE"] == "mysql":
                conn = "mysql+pymysql://%s:%s@%s:3306/%s?charset=utf8mb4" \
                       % ("root", "123456", "localhost", "test")
            else:
                raise
        else:
            if settings["DATABASE"] == "sqlite":
                if sys.platform == "win32":
                    conn = rf"sqlite:///{db_path}?check_same_thread=False"
                elif utils.is_vercel_sqlite():
                    # Vercel production environment is a read-only file system.
                    # See: https://github.com/vercel/community/discussions/314?sort=new
                    # Here are temporary storage solution: copy base_path/data.db to /tmp/data.db
                    # Most containers have a /tmp folder. It's a UNIX convention, and
                    # usually held in memory and cleared on reboot. Don't need to create by yourself.
                    if os.path.exists("/tmp/data.db"):
                        # 当前请求已存在临时存储
                        conn = f"sqlite:////tmp/data.db?check_same_thread=False"
                    elif os.path.exists(db_path):
                        # 当前请求不存在临时存储，但存在github上传的data.db
                        with open(db_path, "rb") as f:
                            binary_content = f.read()
                        with open("/tmp/data.db", "wb") as f:
                            f.write(binary_content)
                        conn = f"sqlite:////tmp/data.db?check_same_thread=False"
                    else:
                        # 此时vercel部署环境还没有data.db，返回异常
                        raise Exception("data.db path empty")
                else:
                    conn = f"sqlite:///{db_path}?check_same_thread=False"
                # conn = "sqlite:///" + BASE_DIR + "/data.db" + "?check_same_thread=False"
            elif settings["DATABASE"] == "mysql":
                mysql_uri = os.environ['MYSQL_URI']
                mysql_uri = mysql_uri.replace("mysql://", "mysql+pymysql://")
                mysql_uri += "?charset=utf8mb4"
                conn = mysql_uri
            else:
                raise
        try:
            engine = create_engine(conn, pool_recycle=-1)
        except:
            raise Exception("MySQL连接失败")
        return engine


def create_all_table():
    engine = SQLEngine()
    models.Model.metadata.create_all(engine)


def db_init():
    engine = SQLEngine()
    Session = sessionmaker(bind=engine)
    session = scoped_session(Session)
    return session