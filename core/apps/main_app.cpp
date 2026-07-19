#include <iostream>
#include <string>
#include <userver/components/minimal_server_component_list.hpp>
#include <userver/components/run.hpp>
#include <userver/ugrpc/server/server_component.hpp>
#include <fmt/format.h>

int main()
{
    const std::string config_path = "../config/config.yaml";

    try
    {
        auto component_list = userver::components::MinimalServerComponentList()
                                  .Append<userver::ugrpc::server::ServerComponent>();

        userver::components::Run(config_path, std::nullopt, std::nullopt, component_list);
        return 0;
    }
    catch (const std::exception &ex)
    {
        std::cerr << fmt::format("[Critical Error] Fail to start daemon: {}\n", ex.what());
        return 1;
    }
}
