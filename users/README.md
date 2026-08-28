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