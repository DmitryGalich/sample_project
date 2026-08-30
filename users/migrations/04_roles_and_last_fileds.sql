ALTER TABLE users 
    ADD COLUMN user_role VARCHAR(10), -- 'customer', 'master', 'admin'
    ADD COLUMN is_active BOOLEAN DEFAULT true,
    ADD COLUMN edited_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN last_login_at TIMESTAMP WITH TIME ZONE;

UPDATE users 
SET user_role = 'customer' 
WHERE user_role IS NULL;

ALTER TABLE users 
    ALTER COLUMN user_role SET NOT NULL;

UPDATE users 
SET is_active = false 
WHERE is_active IS NULL;

ALTER TABLE users 
    ALTER COLUMN is_active SET NOT NULL;
