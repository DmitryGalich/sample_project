INSERT INTO projects (
    id, 
    owner_id,
    title,
    description
) VALUES 
(
    '10000000-1000-0000-0000-000000000000',
    '10000000-0000-0000-0000-000000000000',
    'ivanov ivan project',
    'ivanov ivan project desc'
),
(
    '20000000-1000-0000-0000-000000000000',
    '20000000-0000-0000-0000-000000000000',
    'petrov petr project',
    'petrov petr project desc'
);

INSERT INTO project_team_members (
    project_id, 
    user_id
) VALUES 
(
    '10000000-1000-0000-0000-000000000000',
    '30000000-0000-0000-0000-000000000000'
),
(
    '20000000-1000-0000-0000-000000000000',
    '30000000-0000-0000-0000-000000000000'
)
ON CONFLICT (project_id, user_id) DO NOTHING;
