"""SQLAlchemy engine and session lifecycle helpers."""
from collections.abc import Iterator
from contextlib import contextmanager
from sqlalchemy import create_engine, text
from sqlalchemy.engine import Engine
from sqlalchemy.orm import Session, sessionmaker
from protein3d.config import Settings, get_settings

def build_engine(settings: Settings | None = None) -> Engine:
    config = settings or get_settings()
    return create_engine(config.database_url, pool_pre_ping=True, pool_size=config.database_pool_size, max_overflow=config.database_max_overflow, hide_parameters=True)

def build_session_factory(engine: Engine) -> sessionmaker[Session]:
    return sessionmaker(bind=engine, autoflush=False, expire_on_commit=False)

@contextmanager
def session_scope(factory: sessionmaker[Session]) -> Iterator[Session]:
    session = factory()
    try:
        yield session
        session.commit()
    except Exception:
        session.rollback()
        raise
    finally:
        session.close()

def check_database_ready(engine: Engine) -> bool:
    with engine.connect() as connection:
        return connection.execute(text("SELECT 1")).scalar_one() == 1
