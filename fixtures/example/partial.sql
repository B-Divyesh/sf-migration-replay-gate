-- Simulates an environment where the table landed but the index did not.
CREATE TABLE audit_events (
  id bigint PRIMARY KEY,
  account_id bigint NOT NULL REFERENCES accounts(id)
);

