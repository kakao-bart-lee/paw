import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class PhoneInputField extends StatefulWidget {
  final Function(String) onChanged;
  final String initialCountryCode;

  const PhoneInputField({
    super.key,
    required this.onChanged,
    this.initialCountryCode = '+82',
  });

  @override
  State<PhoneInputField> createState() => _PhoneInputFieldState();
}

class _PhoneInputFieldState extends State<PhoneInputField> {
  late String _selectedCountryCode;
  final TextEditingController _phoneController = TextEditingController();

  final List<String> _countryCodes = ['+82', '+1', '+44', '+81', '+86'];

  @override
  void initState() {
    super.initState();
    _selectedCountryCode = widget.initialCountryCode;
    _phoneController.addListener(_updatePhone);
  }

  @override
  void dispose() {
    _phoneController.removeListener(_updatePhone);
    _phoneController.dispose();
    super.dispose();
  }

  void _updatePhone() {
    final number = _phoneController.text.replaceAll(RegExp(r'[^0-9]'), '');
    if (number.isNotEmpty) {
      widget.onChanged('$_selectedCountryCode$number');
    } else {
      widget.onChanged('');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          height: 52,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Theme.of(context).colorScheme.outline),
          ),
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: DropdownButtonHideUnderline(
            child: DropdownButton<String>(
              value: _selectedCountryCode,
              items: _countryCodes.map((code) {
                return DropdownMenuItem(
                  value: code,
                  child: Text(
                    code,
                    style: Theme.of(context).textTheme.bodyLarge,
                  ),
                );
              }).toList(),
              onChanged: (value) {
                if (value != null) {
                  setState(() {
                    _selectedCountryCode = value;
                    _updatePhone();
                  });
                }
              },
              icon: Icon(
                Icons.arrow_drop_down,
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
              dropdownColor: Theme.of(
                context,
              ).colorScheme.surfaceContainerHighest,
            ),
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: SizedBox(
            height: 52,
            child: TextField(
              key: const ValueKey('phone-input'),
              controller: _phoneController,
              keyboardType: TextInputType.phone,
              inputFormatters: [FilteringTextInputFormatter.digitsOnly],
              style: Theme.of(context).textTheme.bodyLarge,
              decoration: InputDecoration(
                hintText: '전화번호 입력',
                hintStyle: Theme.of(context).textTheme.bodyLarge?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 14,
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
