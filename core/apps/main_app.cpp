#include <iostream>
#include <memory>
#include <string>
#include <format>
#include <version>

#include <grpcpp/grpcpp.h>
#include <grpcpp/ext/proto_server_reflection_plugin.h>
#include "transport.grpc.pb.h"

class MessengerCoreServiceImpl final : public messenger::MessengerCoreService::Service {
    grpc::Status StreamMessages(grpc::ServerContext* context, 
                                grpc::ServerReaderWriter<messenger::Message, messenger::Message>* stream) override {
        messenger::Message message;
        
        std::cout << "[Core Engine] New client connected to StreamMessages." << std::endl;

        // Читаем сообщения, пока клиент держит соединение
        while (stream->Read(&message)) {
            // Используем фичу C++26: placeholder variable (_)
            auto _ = std::format("[Message Received] Chat: {}, Sender: {}, Text: {}", 
                                 message.chat_id(), message.sender_id(), message.content());
            std::cout << _ << std::endl;

            // Меняем статус и отправляем эхо-ответ обратно в стрим
            message.set_status(messenger::MessageStatus::DELIVERED);
            stream->Write(message);
        }

        std::cout << "[Core Engine] Client disconnected." << std::endl;
        return grpc::Status::OK;
    }
};

void RunServer() {
    std::string server_address("0.0.0.0:50051");
    MessengerCoreServiceImpl service;

    grpc::EnableDefaultHealthCheckService(true);
    grpc::reflection::InitProtoReflectionServerBuilderPlugin();
    
    grpc::ServerBuilder builder;
    builder.AddListeningPort(server_address, grpc::InsecureServerCredentials());
    builder.RegisterService(&service);
    
    std::unique_ptr<grpc::Server> server(builder.BuildAndStart());
    std::cout << std::format("[Core Engine] Server listening on: {}", server_address) << std::endl;
    
    server->Wait();
}

int main() {
    std::cout << std::format("[Core Engine] Starting... C++ Standard Macro: {}", __cplusplus) << std::endl;
    RunServer();
    return 0;
}
