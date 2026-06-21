package com.phantom.model

data class Message(
    val messageId: Long,
    val contactId: Long,
    val text: String,
    val direction: Direction,
    val createdAt: Long,
    val deliveredAt: Long?,
    val seenAt: Long?,
)

enum class Direction { Sent, Received }
