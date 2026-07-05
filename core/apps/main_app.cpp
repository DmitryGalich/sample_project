#include <iostream>
#include <print> // Фича C++23/C++26
#include <grpcpp/grpcpp.h>
#include <grpcpp/health_check_service_interface.h>
#include <grpcpp/ext/proto_server_reflection_plugin.h>

int main()
{
    // Фича C++26: Анонимный плейсхолдер "_" (замена неиспользуемой переменной)
    // В C++26 код ниже полностью валиден и не вызывает конфликта имен
    auto _ = "Тестируем синтаксис C++26";
    auto _ = "Второй анонимный плейсхолдер в той же области видимости";

    // Фича C++23/C++26: Прямой вывод через std::print вместо std::cout
    std::print("Инициализация gRPC сервера на стандарте C++26...\n");

    // Инициализация базового gRPC сервера
    grpc::ServerBuilder builder;
    builder.AddListeningPort("0.0.0.0:50051", grpc::InsecureServerCredentials());

    // Включаем рефлексию (полезно для отладки утилитами вроде grpcurl)
    grpc::reflection::InitProtoReflectionServerBuilderPlugin();

    // Переводим инициализацию на стабильный BuildAndStart()
    std::unique_ptr<grpc::Server> server(builder.BuildAndStart());

    if (server)
    {
        std::print("gRPC сервер успешно запущен на порту 50051!\n");
        // Чтобы сервер не завершал работу сразу, обычно вызывают ожидание:
        // server->Wait();
    }
    else
    {
        std::print(stderr, "Ошибка: не удалось создать и запустить gRPC сервер.\n");
        return 1;
    }

    return 0;
}
