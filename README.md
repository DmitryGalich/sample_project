# Sample project

## Development

* Run all except container in vscode
```
docker compose up --scale <excluded_service_name>=0
```

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

* PGADMIN
```
      PGADMIN_DEFAULT_EMAIL: admin@admin.com
      PGADMIN_DEFAULT_PASSWORD: admin
```

* Работа с БД в консоли

```
psql -U admin -d users_db

\dt — показать список всех таблиц.
\l — показать список всех баз данных.
\d имя_таблицы — посмотреть структуру конкретной таблицы (колонки, типы данных).

// НЕ ЗАБЫТЬ ;
SELECT * FROM имя_таблицы; — сделать SQL-запрос (обязательно ставьте ; в конце).

\q — выйти из psql обратно в консоль хоста.

```