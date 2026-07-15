add_library(network_lib 
    src/queue.cpp
)

target_include_directories(network_lib PUBLIC include)
add_library(modules::network_lib ALIAS network_lib)