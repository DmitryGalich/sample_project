#!/bin/bash

# Настройки подключения
KEYCLOAK_URL="http://localhost/auth/realms/sample_project_realm/protocol/openid-connect/token"
CLIENT_ID="frontend"
USERNAME="ivan"

# Запрашиваем пароль в терминале (скрывая ввод, как при sudo)
read -sp "Введите пароль для пользователя '$USERNAME': " PASSWORD
echo ""

echo "⏳ Запрос токена у Keycloak через Nginx..."

# Делаем POST запрос на обмен учетных данных на токен
RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=$CLIENT_ID" \
  -d "username=$USERNAME" \
  -d "password=$PASSWORD" \
  -d "scope=openid")

# Проверяем, есть ли в ответе access_token
if echo "$RESPONSE" | grep -q "access_token"; then
    echo "✅ Токен успешно получен!"
    echo "--------------------------------------------------"
    
    # Вытаскиваем чистую строку токена с помощью встроенного в macOS скрипта python
    ACCESS_TOKEN=$(echo "$RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['access_token'])")
    
    echo "Bearer $ACCESS_TOKEN"
    echo "--------------------------------------------------"
    echo "💡 Скопируйте всю строку выше (включая слово Bearer) и вставьте в Postman."
else
    echo "❌ Ошибка получения токена!"
    # Форматируем ошибку для читаемости с помощью встроенного в macOS json_pp
    echo "$RESPONSE" | json_pp
fi
