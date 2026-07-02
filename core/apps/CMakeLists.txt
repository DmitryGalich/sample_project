set(MAIN_APP_TITLE main_app.cpp)

add_executable(${MAIN_APP_TITLE}
    ${MAIN_APP_TITLE}.cpp
)

target_link_libraries(${MAIN_APP_TITLE} 
    PRIVATE
        gRPC::grpc++
        protobuf::libprotobuf
)