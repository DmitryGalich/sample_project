set(MAIN_APP_TITLE messenger_core)

add_executable(${MAIN_APP_TITLE}
    main_app.cpp
)

# Форсируем проброс путей к заголовкам (headers) через PRIVATE-линковку
target_link_libraries(${MAIN_APP_TITLE} PRIVATE
    userver::core
    userver::grpc
    messenger_proto
)
