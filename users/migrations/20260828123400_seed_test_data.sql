CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

INSERT INTO users (email, display_name) 
VALUES  ('ivanov.ivan@gmail.com', 'ivanov ivan'),
        ('petrov.petr@gmail.com', 'petrov petr'),
        ('maximov.maxim@gmail.com', 'maximov maxim'),
        ('sergeev.sergey@gmail.com', 'sergeev sergey'),
        ('andreev.andrey@gmail.com', 'andreev andrey');
