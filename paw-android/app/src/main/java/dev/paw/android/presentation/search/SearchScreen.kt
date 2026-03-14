package dev.paw.android.presentation.search

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.SmartToy
import androidx.compose.material.icons.filled.Group
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.navigation.PawRoutes
import dev.paw.android.presentation.theme.PawAI
import dev.paw.android.presentation.theme.PawAmber
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface2
import dev.paw.android.presentation.theme.PawSurface3

@Composable
fun SearchScreen(navController: NavController, viewModel: BootstrapViewModel) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    var query by remember { mutableStateOf("") }
    val recentSearches = remember { listOf("김민지", "Paw Assistant", "프로젝트") }

    val filteredChats = if (query.isNotBlank()) {
        uiState.chat.conversations.filter {
            it.name.contains(query, ignoreCase = true)
        }
    } else {
        emptyList()
    }

    Column(
        modifier = Modifier.fillMaxSize().background(PawBackground).statusBarsPadding(),
    ) {
        // Header with search
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            IconButton(onClick = { navController.popBackStack() }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
            TextField(
                value = query,
                onValueChange = { query = it },
                modifier = Modifier.weight(1f),
                placeholder = { Text("메시지, 사용자, Agent 검색") },
                leadingIcon = { Icon(Icons.Filled.Search, null, tint = PawMutedText) },
                trailingIcon = {
                    if (query.isNotBlank()) {
                        IconButton(onClick = { query = "" }) {
                            Icon(Icons.Filled.Close, null, tint = PawMutedText)
                        }
                    }
                },
                singleLine = true,
                shape = RoundedCornerShape(999.dp),
                colors = TextFieldDefaults.colors(
                    focusedContainerColor = PawSurface1,
                    unfocusedContainerColor = PawSurface1,
                    focusedTextColor = PawStrongText,
                    unfocusedTextColor = PawStrongText,
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent,
                    cursorColor = PawPrimary,
                ),
            )
        }

        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
        ) {
            if (query.isNotBlank()) {
                if (filteredChats.isNotEmpty()) {
                    item {
                        Text("검색 결과", style = MaterialTheme.typography.labelSmall, color = PawMutedText, modifier = Modifier.padding(vertical = 8.dp))
                    }
                    items(filteredChats) { chat ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable {
                                    viewModel.chatViewModel.selectConversation(chat.id)
                                    navController.navigate(PawRoutes.chatDetail(chat.id))
                                }
                                .padding(vertical = 12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            Box(
                                modifier = Modifier.size(48.dp).background(PawSurface3, CircleShape),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(chat.name.first().toString(), style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                            }
                            Column(modifier = Modifier.weight(1f)) {
                                Text(chat.name, style = MaterialTheme.typography.titleMedium, color = PawStrongText, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text(chat.lastMessage ?: "", style = MaterialTheme.typography.bodySmall, color = PawMutedText, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    }
                } else {
                    item {
                        Column(
                            modifier = Modifier.fillMaxWidth().padding(vertical = 64.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Box(modifier = Modifier.size(64.dp).background(PawSurface2, CircleShape), contentAlignment = Alignment.Center) {
                                Icon(Icons.Filled.Search, null, tint = PawMutedText, modifier = Modifier.size(32.dp))
                            }
                            Text("검색 결과가 없습니다", style = MaterialTheme.typography.titleMedium, color = PawStrongText, modifier = Modifier.padding(top = 16.dp))
                            Text("다른 키워드로 검색해보세요", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                        }
                    }
                }
            } else {
                // Recent searches
                item {
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text("최근 검색", style = MaterialTheme.typography.labelSmall, color = PawMutedText)
                        Text("모두 삭제", style = MaterialTheme.typography.labelSmall, color = PawPrimary)
                    }
                }
                items(recentSearches) { search ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { query = search }
                            .padding(vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Icon(Icons.Filled.History, null, tint = PawMutedText, modifier = Modifier.size(20.dp))
                        Text(search, style = MaterialTheme.typography.bodyMedium, color = PawStrongText)
                    }
                }

                // Quick shortcuts
                item {
                    Text("바로가기", style = MaterialTheme.typography.labelSmall, color = PawMutedText, modifier = Modifier.padding(top = 24.dp, bottom = 8.dp))
                }
                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .background(PawSurface1, RoundedCornerShape(12.dp))
                                .clickable { navController.navigate(PawRoutes.AGENT_HUB) }
                                .padding(16.dp),
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                Box(modifier = Modifier.size(40.dp).background(PawAI.copy(alpha = 0.1f), RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
                                    Icon(Icons.Filled.SmartToy, null, tint = PawAI, modifier = Modifier.size(20.dp))
                                }
                                Text("Agent 찾기", style = MaterialTheme.typography.labelLarge, color = PawStrongText)
                            }
                        }
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .background(PawSurface1, RoundedCornerShape(12.dp))
                                .clickable { navController.navigate(PawRoutes.GROUP_CREATE) }
                                .padding(16.dp),
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                Box(modifier = Modifier.size(40.dp).background(PawAmber.copy(alpha = 0.1f), RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
                                    Icon(Icons.Filled.Group, null, tint = PawAmber, modifier = Modifier.size(20.dp))
                                }
                                Text("그룹 생성", style = MaterialTheme.typography.labelLarge, color = PawStrongText)
                            }
                        }
                    }
                }
            }
        }
    }
}
