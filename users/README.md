* Пример адреса в Postman
```
localhost:8080 | UserService/GetAllUsers
```

* Пример проверки метода через grpcurl
```
grpcurl -plaintext \
  -import-path /Users/dmitry/Projects/sample_project/users/proto \
  -proto users.proto \
  -d '{"email":"test1@example.com","display_name":"Test1"}' \
  localhost:8080 \
  users.UsersService/AddUser
```

* Применение миграций
```
sqlx migrate run --database-url $DATABASE_URL
```

* Работа с ЬД в консоли

```
psql -U admin -d users_db

\dt — показать список всех таблиц.
\l — показать список всех баз данных.
\d имя_таблицы — посмотреть структуру конкретной таблицы (колонки, типы данных).
SELECT * FROM имя_таблицы; — сделать SQL-запрос (обязательно ставьте ; в конце).
\q — выйти из psql обратно в консоль хоста.

```