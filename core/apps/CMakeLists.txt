set(MAIN_APP_TITLE main_app)

add_executable(${MAIN_APP_TITLE}
    ${MAIN_APP_TITLE}.cpp
)

target_generate_grpc_protos(
    TARGET     ${MAIN_APP_TITLE}
    PROTO_FILE "transport.proto"
    SRC_DIR    ${PROTO_SRC_DIR}
    OUT_DIR    ${PROTO_OUT_DIR}
)

target_link_libraries(${MAIN_APP_TITLE} 
    PRIVATE
        gRPC::grpc++
        gRPC::grpc++_reflection
        protobuf::libprotobuf
)