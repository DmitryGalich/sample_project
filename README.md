# Sample project

## Development

### Steps if creating new service

* Create backend container
* Create db container
* Create instance in Nginx
* Create proto file
* Create migrations 
* [Apply migrations](#apply-migrations)
* [Check in terminal inside database container](#simple-work-with-db)
* [Check db with pgadmin](#pgadmin)
* Make request handling functions in service
* [Test service handling functions with Postman](#postman-address-example)

### Cookbook

#### Run all except container in vscode
```
docker compose up --scale <excluded_service_name>=0
```

#### Postman address example
```
localhost:8080 | UserService/GetAllUsers
```

#### Example of checking grpc function by grpcurl
```
grpcurl -plaintext \
  -import-path /Users/dmitry/Projects/sample_project/users/proto \
  -proto users.proto \
  -d '{"email":"test1@example.com","display_name":"Test1"}' \
  localhost:8080 \
  users.UsersService/AddUser
```

#### Apply migrations
```
sqlx migrate run --database-url $DATABASE_URL
```

#### PGADMIN
```
      PGADMIN_DEFAULT_EMAIL: admin@admin.com
      PGADMIN_DEFAULT_PASSWORD: admin
```

#### Simple work with db 

```
psql -U admin -d users_db

\dt — all tables
\l — all databases
\d table_title — db structure

// DO NOT FORGET ;

SELECT * FROM table_title; — request example;

// DO NOT FORGET ;

\q — quit

```