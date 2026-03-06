import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../providers/auth_provider.dart';

class DeviceNameScreen extends ConsumerStatefulWidget {
  const DeviceNameScreen({super.key});

  @override
  ConsumerState<DeviceNameScreen> createState() => _DeviceNameScreenState();
}

class _DeviceNameScreenState extends ConsumerState<DeviceNameScreen> {
  final TextEditingController _nameController = TextEditingController();
  bool _isNameValid = false;

  @override
  void initState() {
    super.initState();
    // Default device name (can be improved with device_info_plus later)
    _nameController.text = 'My Device';
    _isNameValid = _nameController.text.isNotEmpty;
    _nameController.addListener(_validateName);
  }

  @override
  void dispose() {
    _nameController.removeListener(_validateName);
    _nameController.dispose();
    super.dispose();
  }

  void _validateName() {
    setState(() {
      _isNameValid = _nameController.text.trim().isNotEmpty;
    });
  }

  void _onSubmit() async {
    if (!_isNameValid) return;

    await ref.read(authNotifierProvider.notifier).setDeviceName(_nameController.text.trim());
    
    if (mounted) {
      final state = ref.read(authNotifierProvider);
      if (state.step == AuthStep.authenticated) {
        // Navigate to main app shell
        context.go('/chat');
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authNotifierProvider);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const SizedBox(height: 16),
              Text(
                '이 기기 이름 설정',
                style: Theme.of(context).textTheme.headlineLarge,
              ),
              const SizedBox(height: 8),
              Text(
                '다른 기기에서 이 기기를 식별하는 데 사용됩니다',
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
              ),
              const SizedBox(height: 32),
              TextField(
                controller: _nameController,
                style: Theme.of(context).textTheme.bodyLarge,
                decoration: InputDecoration(
                  hintText: '기기 이름 입력',
                  hintStyle: Theme.of(context).textTheme.bodyLarge?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 14,
                  ),
                ),
                onSubmitted: (_) => _onSubmit(),
              ),
              const SizedBox(height: 32),
              SizedBox(
                width: double.infinity,
                height: 52,
                child: FilledButton(
                  onPressed: _isNameValid && !authState.isLoading
                      ? _onSubmit
                      : null,
                  style: FilledButton.styleFrom(
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                  child: authState.isLoading
                      ? const SizedBox(
                          width: 24,
                          height: 24,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text(
                          '시작하기',
                          style: TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
