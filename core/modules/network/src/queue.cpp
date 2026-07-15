#include <queue.hpp>

template <typename T, size_t  Capacity>
LockFreeQueue<T, Capacity>::LockFreeQueue() : buffer_(Capacity) {};

template <typename T, size_t  Capacity>
bool LockFreeQueue<T, Capacity>::push(const T &item)
{

    size_t current_tail = tail_.load(std::memory_order_relaxed);
    size_t next_tail = (current_tail + 1) % Capacity;

    if (next_tail == head_.load(std::memory_order_acquire))
    {
        return false;
    }

    if (buffer_[current_tail].is_ready.load(std::memory_order_relaxed))
    {
        return false;
    }

    buffer_[current_tail].data = item;
    buffer_[current_tail].is_ready.store(true, std::memory_order_release);
    tail_.store(next_tail, std::memory_order_release);
    return true;
}

template <typename T, size_t  Capacity>
bool LockFreeQueue<T, Capacity>::pop(T &item)
{
    size_t current_head = head_.load(std::memory_order_relaxed);

    if (current_head == tail_.load(std::memory_order_acquire))
    {
        return false;
    }

    if (!buffer_[current_head].is_ready.load(std::memory_order_acquire))
    {
        return false;
    }

    item = buffer_[current_head].data;
    buffer_[current_head].is_ready.store(false, std::memory_order_release);
    head_.store((current_head + 1) % Capacity, std::memory_order_release);
    return true;
}