function(TARGET_GENERATE_GRPC_PROTOS)
    set(options)
    set(oneValueArgs TARGET PROTO_FILE SRC_DIR OUT_DIR)
    set(multiValueArgs)
    cmake_parse_arguments(GEN "${options}" "${oneValueArgs}" "${multiValueArgs}" ${ARGN})

    # Проверяем, что все обязательные аргументы переданы
    if(NOT GEN_PROTO_FILE OR NOT GEN_SRC_DIR OR NOT GEN_OUT_DIR OR NOT GEN_TARGET)
        message(FATAL_ERROR "target_generate_grpc_protos: Missing required arguments!")
    endif()

    # Получаем имя файла без расширения (например, "transport")
    get_filename_component(PROTO_NAME ${GEN_PROTO_FILE} NAME_WE)

    set(GENERATED_CC "${GEN_OUT_DIR}/${PROTO_NAME}.pb.cc")
    set(GENERATED_H "${GEN_OUT_DIR}/${PROTO_NAME}.pb.h")
    set(GENERATED_GRPC_CC "${GEN_OUT_DIR}/${PROTO_NAME}.grpc.pb.cc")
    set(GENERATED_GRPC_H "${GEN_OUT_DIR}/${PROTO_NAME}.grpc.pb.h")

    # Поиск утилит компиляции (они в /usr/local/bin)
    find_program(PROTOC_PROGRAM protoc REQUIRED)
    find_program(GRPC_CPP_PLUGIN_PROGRAM grpc_cpp_plugin REQUIRED)

    # Кастомная команда генерации
    add_custom_command(
        OUTPUT "${GENERATED_CC}" "${GENERATED_H}" "${GENERATED_GRPC_CC}" "${GENERATED_GRPC_H}"
        COMMAND ${PROTOC_PROGRAM}
        ARGS --cpp_out=${GEN_OUT_DIR} --grpc_out=${GEN_OUT_DIR}
             --plugin=protoc-gen-grpc=${GRPC_CPP_PLUGIN_PROGRAM}
             -I${GEN_SRC_DIR} ${GEN_SRC_DIR}/${GEN_PROTO_FILE}
        DEPENDS ${GEN_SRC_DIR}/${GEN_PROTO_FILE}
        COMMENT "Generating gRPC/Protobuf sources for ${PROTO_NAME}..."
    )

    # Привязываем сгенерированные файлы к конкретной цели (таргету)
    target_sources(${GEN_TARGET} PRIVATE
        "${GENERATED_CC}"
        "${GENERATED_GRPC_CC}"
    )

    # Добавляем папку с заголовками в инклюды таргета
    target_include_directories(${GEN_TARGET} PUBLIC ${GEN_OUT_DIR})
endfunction()
