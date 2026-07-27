-- Role passwords and login policy are owned by yuniruyuni.net/NixOS.
-- pgschema evaluates this block only while validating the desired schema in
-- its plan database; it does not emit role DDL for the target database.
-- Production and test targets must therefore create the app role before apply.
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'fighter_app') THEN
    CREATE ROLE fighter_app;
  END IF;
END
$$;
