ALTER TABLE users
    ALTER COLUMN user_role SET DEFAULT 'customer',
    ALTER COLUMN is_active SET DEFAULT true;

ALTER TABLE users
    ADD CONSTRAINT users_user_role_check
    CHECK (user_role IN ('customer', 'master', 'admin'));

ALTER TABLE users
    ADD CONSTRAINT users_password_hash_not_placeholder_check
    CHECK (password_hash <> 'CHANGE_ME_IN_FUTURE');

CREATE UNIQUE INDEX users_email_lower_unique_idx
    ON users (LOWER(email));
