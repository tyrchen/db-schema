--- The following test tables are created by ChatGPT
CREATE SCHEMA gpt;

CREATE TYPE gpt.login_method AS ENUM (
  'email',
  'google',
  'github'
);

CREATE TYPE gpt.post_status AS ENUM (
  'draft',
  'published',
  'archived'
);

CREATE TYPE gpt.address AS (
  street varchar ( 255),
  city VARCHAR(100),
  state CHAR(2),
  postal_code CHAR(5));

CREATE TABLE gpt.users (
  id serial PRIMARY KEY,
  username varchar(50) UNIQUE NOT NULL,
  email varchar(255) UNIQUE NOT NULL,
  first_name varchar(50),
  last_name varchar(50),
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE gpt.logins (
  id serial PRIMARY KEY,
  user_id integer NOT NULL REFERENCES gpt.users (id) ON DELETE CASCADE,
  method gpt.login_method NOT NULL,
  identifier varchar(255) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW(),
  UNIQUE (method, identifier)
);

CREATE TABLE gpt.posts (
  id serial PRIMARY KEY,
  user_id integer NOT NULL REFERENCES gpt.users (id) ON DELETE CASCADE,
  title varchar(255) NOT NULL,
  content text NOT NULL,
  status gpt.post_status NOT NULL DEFAULT 'draft',
  published_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE TABLE gpt.comments (
  id serial PRIMARY KEY,
  post_id integer NOT NULL REFERENCES gpt.posts (id) ON DELETE CASCADE,
  user_id integer NOT NULL REFERENCES gpt.users (id) ON DELETE CASCADE,
  content text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

-- create indexes
CREATE INDEX user_posts ON gpt.posts (user_id, created_at DESC);

-- create views
CREATE VIEW gpt.posts_with_comments AS
SELECT
  p.id,
  p.user_id,
  p.title,
  p.content,
  p.status,
  p.published_at,
  p.created_at,
  p.updated_at,
  json_agg(json_build_object('id', c.id, 'user_id', c.user_id, 'content', c.content, 'created_at', c.created_at, 'updated_at', c.updated_at)) AS comments
FROM
  gpt.posts p
  LEFT JOIN gpt.comments c ON c.post_id = p.id
GROUP BY
  p.id;

CREATE MATERIALIZED VIEW gpt.users_with_posts AS
SELECT
  u.id,
  u.username,
  u.email,
  u.first_name,
  u.last_name,
  u.created_at,
  u.updated_at,
  json_agg(json_build_object('id', p.id, 'title', p.title, 'content', p.content, 'status', p.status, 'published_at', p.published_at, 'created_at', p.created_at, 'updated_at', p.updated_at)) AS posts
FROM
  gpt.users u
  LEFT JOIN gpt.posts p ON p.user_id = u.id
GROUP BY
  u.id;

-- create function
CREATE OR REPLACE FUNCTION gpt.refresh_users_with_posts ()
  RETURNS TRIGGER
  AS $$
BEGIN
  REFRESH MATERIALIZED VIEW gpt.users_with_posts;
  RETURN NULL;
END;
$$
LANGUAGE plpgsql;

-- create triggers
CREATE TRIGGER refresh_users_with_posts
  AFTER INSERT OR UPDATE OR DELETE ON gpt.posts
  FOR EACH STATEMENT
  EXECUTE PROCEDURE gpt.refresh_users_with_posts ();
