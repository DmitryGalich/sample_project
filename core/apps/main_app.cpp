// #include <print>

// int main() {
//     std::println("Hello from Core");
//     return 0;
// }

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

//     1  cd ..
//     2  ls
//     3  cd tmp/
//     4  ls
//     5  cd grpc-repo/
//     6  ls
//     7  mkdir build
//     8  cd build
//     9  cmake -DgRPC_INSTALL=ON              -DgRPC_BUILD_TESTS=OFF              -DCMAKE_BUILD_TYPE=Release              ..
//    10  cmake --version
//    11  cmake -DgRPC_INSTALL=ON              -DgRPC_BUILD_TESTS=OFF              -DCMAKE_BUILD_TYPE=Release              -DCMAKE_POLICY_VERSION_MINIMUM=3.5              ..
//    12  make -j$(nproc)
//    13  make install
//    14  cd ..
//    15  cd workspace
//    16  ls
//    17  cd ..
//    18  ls
//    19  cd workspaces
//    20  cd sample_project/
//    21  history