-- 1. Добавляем все новые поля (password_hash временно разрешаем делать NULL)
ALTER TABLE users 
    ADD COLUMN password_hash VARCHAR(255),
    ADD COLUMN first_name VARCHAR(100),
    ADD COLUMN last_name VARCHAR(100),
    ADD COLUMN avatar_url VARCHAR(500),
    ADD COLUMN phone VARCHAR(50),
    ADD COLUMN bio TEXT;

-- 2. Обновляем существующих пользователей (ставим им временную заглушку вместо пароля)
-- ВАЖНО: Замените 'CHANGE_ME_IN_FUTURE' на реальный хэш или заставьте пользователей сбросить пароль
UPDATE users 
SET password_hash = 'CHANGE_ME_IN_FUTURE' 
WHERE password_hash IS NULL;

-- 3. Теперь, когда у всех есть значение, делаем поле NOT NULL
ALTER TABLE users 
    ALTER COLUMN password_hash SET NOT NULL;
