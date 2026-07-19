#include <iostream>
#include <string>
#include <userver/components/minimal_server_component_list.hpp>
#include <userver/components/run.hpp> 
#include <userver/ugrpc/server/server_component.hpp>
#include <fmt/format.h>

int main(int argc, char* argv[]) {
    std::cout << "[Core Engine] Initializing userver v3.1. Mode: C++20\n";

    // Нам обязательно нужен путь к файлу конфигурации
    if (argc < 2) {
        std::cerr << "Error: Missing config path!\n";
        std::cerr << "Usage: ./messenger_core <path/to/config.yaml>\n";
        return 1;
    }

    // Извлекаем путь к config.yaml из аргументов командной строки
    std::string config_path = argv[1];
    std::cout << fmt::format("[Core Engine] Loading configuration from: {}\n", config_path);

    try {
        // Собираем минимальный набор системных компонентов Яндекса + gRPC сервер
        auto component_list = userver::components::MinimalServerComponentList()
                                  .Append<userver::ugrpc::server::ServerComponent>();

        // Сигнатура v3.1: первым аргументом передается строка пути к файлу конфигурации
        userver::components::Run(config_path, std::nullopt, std::nullopt, component_list);
        return 0;
    } 
    catch (const std::exception& ex) {
        std::cerr << fmt::format("[Critical Error] Fail to start daemon: {}\n", ex.what());
        return 1;
    }
}
