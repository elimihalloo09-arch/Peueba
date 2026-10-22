-- Sin Docker: crea en tu PostgreSQL el usuario y la base que usa la app.
--   psql -U postgres -f preparar-bd.sql
-- Luego, en backend/.env, usa el puerto de tu PostgreSQL (normalmente 5432):
--   DATABASE_URL=postgres://avances:avances@localhost:5432/avances
CREATE ROLE avances LOGIN PASSWORD 'avances';
CREATE DATABASE avances OWNER avances;
