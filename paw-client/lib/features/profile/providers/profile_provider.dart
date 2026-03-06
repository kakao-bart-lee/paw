import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:get_it/get_it.dart';

import '../../../core/http/api_client.dart';

class ProfileState {
  final AsyncValue<Map<String, dynamic>> userAsync;
  final bool isSaving;

  const ProfileState({
    this.userAsync = const AsyncValue.loading(),
    this.isSaving = false,
  });

  ProfileState copyWith({
    AsyncValue<Map<String, dynamic>>? userAsync,
    bool? isSaving,
  }) {
    return ProfileState(
      userAsync: userAsync ?? this.userAsync,
      isSaving: isSaving ?? this.isSaving,
    );
  }
}

class ProfileNotifier extends StateNotifier<ProfileState> {
  final ApiClient _apiClient;

  ProfileNotifier(this._apiClient) : super(const ProfileState()) {
    loadProfile();
  }

  Future<void> loadProfile() async {
    state = state.copyWith(userAsync: const AsyncValue.loading());
    try {
      final result = await _apiClient.getMe();
      state = state.copyWith(userAsync: AsyncValue.data(result));
    } catch (e, st) {
      state = state.copyWith(userAsync: AsyncValue.error(e, st));
    }
  }

  Future<void> updateProfile(String displayName) async {
    state = state.copyWith(isSaving: true);
    try {
      await _apiClient.updateMe(displayName: displayName);
      await loadProfile();
    } catch (e, st) {
      state = state.copyWith(
        userAsync: AsyncValue.error(e, st),
        isSaving: false,
      );
      return;
    }
    state = state.copyWith(isSaving: false);
  }
}

final profileProvider =
    StateNotifierProvider<ProfileNotifier, ProfileState>((ref) {
  final apiClient = GetIt.instance<ApiClient>();
  return ProfileNotifier(apiClient);
});
