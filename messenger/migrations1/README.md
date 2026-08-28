* Add user to project
```
INSERT INTO project_members (project_id, user_id, project_role) 
VALUES ('project-uuid-здесь', 'user-uuid-здесь', 'editor');
```

* Get all projects with user
```
SELECT p.*, pm.project_role 
FROM projects p
JOIN project_members pm ON p.id = pm.project_id
WHERE pm.user_id = 'user-uuid-здесь';
```

* Get users and names from project
```
SELECT u.id, u.username, pm.project_role 
FROM project_members pm
JOIN users u ON pm.user_id = u.id
WHERE pm.project_id = 'project-uuid-здесь';
```
