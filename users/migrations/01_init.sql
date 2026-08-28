CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_role') THEN
        CREATE TYPE user_role AS ENUM ('customer', 'master', 'admin');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url VARCHAR(500),
    phone VARCHAR(50),
    bio TEXT,
    
    role user_role NOT NULL DEFAULT 'customer',
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    last_login_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    
    -- -- Индексы для быстрого поиска
    -- CONSTRAINT users_email_active_unique UNIQUE (email) 
    --     WHERE deleted_at IS NULL
);

-- -- Индексы для оптимизации запросов
-- CREATE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
-- CREATE INDEX idx_users_display_name ON users(display_name) WHERE deleted_at IS NULL;
-- CREATE INDEX idx_users_role ON users(role) WHERE deleted_at IS NULL;
-- CREATE INDEX idx_users_is_active ON users(is_active) WHERE deleted_at IS NULL;
-- CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NOT NULL;

-- -- Индекс для полнотекстового поиска
-- CREATE INDEX idx_users_search ON users USING GIN (
--     to_tsvector('english', 
--         COALESCE(display_name, '') || ' ' || 
--         COALESCE(first_name, '') || ' ' || 
--         COALESCE(last_name, '') || ' ' || 
--         COALESCE(email, '')
--     )
-- ) WHERE deleted_at IS NULL;

-- -- Триггер для автоматического обновления edited_at
-- CREATE OR REPLACE FUNCTION update_edited_at_column()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     NEW.edited_at = NOW();
--     RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- CREATE TRIGGER trigger_update_users_edited_at
--     BEFORE UPDATE ON users
--     FOR EACH ROW
--     EXECUTE FUNCTION update_edited_at_column();

-- Комментарии к таблице и полям
COMMENT ON TABLE users IS 'Таблица пользователей системы';
COMMENT ON COLUMN users.id IS 'Уникальный идентификатор пользователя (UUID)';
COMMENT ON COLUMN users.email IS 'Email пользователя (уникальный)';
COMMENT ON COLUMN users.display_name IS 'Отображаемое имя пользователя';
COMMENT ON COLUMN users.password_hash IS 'Хешированный пароль пользователя';
COMMENT ON COLUMN users.first_name IS 'Имя пользователя';
COMMENT ON COLUMN users.last_name IS 'Фамилия пользователя';
COMMENT ON COLUMN users.avatar_url IS 'URL аватара пользователя';
COMMENT ON COLUMN users.phone IS 'Номер телефона';
COMMENT ON COLUMN users.bio IS 'Краткая биография пользователя';
COMMENT ON COLUMN users.role IS 'Роль пользователя: customer, worker, admin';
COMMENT ON COLUMN users.is_active IS 'Активен ли аккаунт';
COMMENT ON COLUMN users.created_at IS 'Дата и время создания записи';
COMMENT ON COLUMN users.edited_at IS 'Дата и время последнего обновления';
COMMENT ON COLUMN users.deleted_at IS 'Дата и время мягкого удаления';
COMMENT ON COLUMN users.last_login_at IS 'Дата и время последнего входа';