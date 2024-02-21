CREATE TABLE possible_codes (
    code VARCHAR(31) PRIMARY KEY
);

INSERT INTO possible_codes (code)
SELECT LPAD(generate_series::text, 4, '0')
FROM generate_series(0, 9999);