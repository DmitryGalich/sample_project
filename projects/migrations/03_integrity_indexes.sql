CREATE INDEX projects_owner_id_idx
    ON projects(owner_id);

CREATE INDEX project_team_members_user_id_idx
    ON project_team_members(user_id);

CREATE INDEX projects_active_created_at_idx
    ON projects(created_at DESC)
    WHERE deleted_at IS NULL;
