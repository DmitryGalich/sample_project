#pragma once

#include <atomic>
#include <cstddef>
#include <vector>

template <typename T, size_t Capacity>
class LockFreeQueue
{
private:
    struct Node
    {
        T data;
        std::atomic<bool> is_ready{false};
    };

    std::vector<Node> buffer_;
    std::atomic<size_t> head_{0};
    std::atomic<size_t> tail_{0};

public:
    LockFreeQueue();

    bool push(const T &item);
    bool pop(T &item);
};