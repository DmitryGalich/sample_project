#!/bin/bash
set -e

echo "Ожидание готовности PostgreSQL (database:5432)..."
# Скрипт пингует порт базы, пока он не откроется
while ! nc -z database 5432; do
  sleep 0.5
done

echo "PostgreSQL запущен. Передаем управление в CMD."
exec "$@"
