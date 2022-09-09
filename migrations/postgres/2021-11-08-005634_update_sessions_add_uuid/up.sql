-- drop all sessions before creating NOT NULL column
DELETE FROM user_sessions;

ALTER TABLE user_sessions ADD COLUMN uuid UUID NOT NULL;
CREATE UNIQUE INDEX unique_user_sessions_uuid ON user_sessions(uuid);
