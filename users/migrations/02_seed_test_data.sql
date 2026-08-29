INSERT INTO users (
    id,
    email,
    display_name,
    password_hash,
    first_name,
    last_name,
    avatar_url,
    phone,
    bio,
    role,
    is_active,
    created_at,
    edited_at,
    last_login_at,
    deleted_at -- Добавили столбец сюда
) VALUES 

-- Администратор
(
    '11111111-1111-1111-1111-111111111111',
    'admin@example.com',
    'Admin User',
    'hashed_password_123',
    'Admin',
    'System',
    'https://example.com/avatars/admin.jpg',
    '+1234567890',
    'System administrator',
    'admin',
    true,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '1 hour',
    NULL -- Добавили NULL для deleted_at
),

-- Активные пользователи (customer)
(
    '22222222-2222-2222-2222-222222222222',
    'john@example.com',
    'John Doe',
    'hashed_password_123',
    'John',
    'Doe',
    'https://example.com/avatars/john.jpg',
    '+1234567891',
    'Software developer and open source contributor',
    'customer',
    true,
    NOW() - INTERVAL '20 days',
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '2 hours',
    NULL
),
(
    '33333333-3333-3333-3333-333333333333',
    'jane@example.com',
    'Jane Smith',
    'hashed_password_123',
    'Jane',
    'Smith',
    'https://example.com/avatars/jane.jpg',
    '+1234567892',
    'UX/UI designer passionate about accessibility',
    'customer',
    true,
    NOW() - INTERVAL '15 days',
    NOW() - INTERVAL '5 days',
    NOW() - INTERVAL '1 day',
    NULL
),
(
    '44444444-4444-4444-4444-444444444444',
    'bob@example.com',
    'Bob Johnson',
    'hashed_password_123',
    'Bob',
    'Johnson',
    'https://example.com/avatars/bob.jpg',
    '+1234567893',
    'DevOps engineer, cloud architect',
    'customer',
    true,
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '3 days',
    NOW() - INTERVAL '3 hours',
    NULL
),

-- Активные пользователи (master)
(
    '55555555-5555-5555-5555-555555555555',
    'alice@example.com',
    'Alice master',
    'hashed_password_123',
    'Alice',
    'master',
    'https://example.com/avatars/alice.jpg',
    '+1234567894',
    'Senior developer, team lead',
    'master',
    true,
    NOW() - INTERVAL '25 days',
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '4 hours',
    NULL
),
(
    '66666666-6666-6666-6666-666666666666',
    'mike@example.com',
    'Mike Technician',
    'hashed_password_123',
    'Mike',
    'Technician',
    'https://example.com/avatars/mike.jpg',
    '+1234567895',
    'Hardware specialist, repair technician',
    'master',
    true,
    NOW() - INTERVAL '18 days',
    NOW() - INTERVAL '8 days',
    NOW() - INTERVAL '5 hours',
    NULL
),

-- Неактивный пользователь
(
    '77777777-7777-7777-7777-777777777777',
    'sarah@example.com',
    'Sarah Inactive',
    'hashed_password_123',
    'Sarah',
    'Inactive',
    NULL,
    '+1234567896',
    'Temporarily inactive account',
    'customer',
    false,
    NOW() - INTERVAL '40 days',
    NOW() - INTERVAL '20 days',
    NOW() - INTERVAL '30 days',
    NULL
),

-- Мягко удаленный пользователь
(
    '88888888-8888-8888-8888-888888888888',
    'deleted@example.com',
    'Deleted User',
    'hashed_password_123',
    'Deleted',
    'User',
    NULL,
    '+1234567897',
    'This user was soft deleted',
    'customer',
    false,
    NOW() - INTERVAL '50 days',
    NOW() - INTERVAL '40 days',
    NOW() - INTERVAL '35 days',
    NOW() - INTERVAL '5 days'
);


-- -- Вставляем дополнительные пользователи для тестирования пагинации
-- INSERT INTO users (
--     email,
--     display_name,
--     password_hash,
--     first_name,
--     last_name,
--     role,
--     is_active,
--     created_at
-- )
-- SELECT 
--     'user' || i || '@example.com',
--     'User ' || i,
--     'hashed_password_123',
--     'First' || i,
--     'Last' || i,
--     CASE 
--         WHEN i % 3 = 0 THEN 'admin'::user_role
--         WHEN i % 3 = 1 THEN 'master'::user_role
--         ELSE 'customer'::user_role
--     END,
--     CASE WHEN i % 5 = 0 THEN false ELSE true END,
--     NOW() - (i || ' days')::INTERVAL
-- FROM generate_series(1, 20) AS i;

-- -- Создаем представление для активных пользователей (опционально)
-- CREATE OR REPLACE VIEW active_users AS
-- SELECT * FROM users 
-- WHERE deleted_at IS NULL AND is_active = true;

-- -- Создаем представление для статистики (опционально)
-- CREATE OR REPLACE VIEW user_stats AS
-- SELECT 
--     role,
--     COUNT(*) as total,
--     COUNT(*) FILTER (WHERE is_active = true) as active,
--     COUNT(*) FILTER (WHERE deleted_at IS NOT NULL) as deleted,
--     MIN(created_at) as earliest_created,
--     MAX(created_at) as latest_created
-- FROM users
-- GROUP BY role;

-- -- Функция для поиска пользователей (опционально)
-- CREATE OR REPLACE FUNCTION search_users(search_term TEXT)
-- RETURNS TABLE (
--     id UUID,
--     display_name VARCHAR,
--     email VARCHAR,
--     role user_role,
--     relevance FLOAT4
-- ) AS $$
-- BEGIN
--     RETURN QUERY
--     SELECT 
--         u.id,
--         u.display_name,
--         u.email,
--         u.role,
--         ts_rank(
--             to_tsvector('english', 
--                 COALESCE(display_name, '') || ' ' || 
--                 COALESCE(first_name, '') || ' ' || 
--                 COALESCE(last_name, '') || ' ' || 
--                 COALESCE(email, '')
--             ),
--             plainto_tsquery('english', search_term)
--         ) as relevance
--     FROM users u
--     WHERE 
--         deleted_at IS NULL
--         AND to_tsvector('english', 
--             COALESCE(display_name, '') || ' ' || 
--             COALESCE(first_name, '') || ' ' || 
--             COALESCE(last_name, '') || ' ' || 
--             COALESCE(email, '')
--         ) @@ plainto_tsquery('english', search_term)
--     ORDER BY relevance DESC
--     LIMIT 50;
-- END;
-- $$ LANGUAGE plpgsql;