-- Add migration script here

CREATE FUNCTION update_at_timestamp() RETURNS trigger AS $update_at_timestamp$
	BEGIN
  	NEW.updated_at = CURRENT_TIMESTAMP;
  	RETURN NEW;
	END;
$update_at_timestamp$ LANGUAGE plpgsql;

CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
	  user_iid VARCHAR(64) NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE TRIGGER user_update_at BEFORE INSERT OR UPDATE ON users 
    FOR EACH ROW EXECUTE FUNCTION update_at_timestamp();

-- Table to capture user settings/preferences
CREATE TABLE user_settings (
    user_id      INT PRIMARY KEY,
    is_dark_mode BOOLEAN DEFAULT false NOT NULL,
    language VARCHAR(10) DEFAULT 'en' NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

