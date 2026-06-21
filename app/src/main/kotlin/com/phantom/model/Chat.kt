package com.phantom.model

data class Chat(
    val contactId: Long,
    val alias: String,
    val onionAddress: String,
    val lastMessage: String?,
    val lastMessageAt: Long,
    val unreadCount: Int,
    val isOnline: Boolean = false,
)
