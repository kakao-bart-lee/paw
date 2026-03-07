import '../http/api_client.dart';

enum UiErrorCode {
  unauthorized,
  forbidden,
  server,
  network,
  timeout,
  client,
  unknown,
}

class UiError {
  final UiErrorCode code;
  final String message;

  const UiError({required this.code, required this.message});

  bool get shouldForceLogin => code == UiErrorCode.unauthorized;
}

class AppErrorMapper {
  static UiError map(Object error) {
    if (error is ApiException) {
      switch (error.kind) {
        case ApiErrorKind.unauthorized:
          return const UiError(
            code: UiErrorCode.unauthorized,
            message: '세션이 만료되었습니다. 다시 로그인해주세요.',
          );
        case ApiErrorKind.forbidden:
          return const UiError(
            code: UiErrorCode.forbidden,
            message: '접근 권한이 없습니다.',
          );
        case ApiErrorKind.server:
          return const UiError(
            code: UiErrorCode.server,
            message: '서버에 일시적인 문제가 있습니다. 잠시 후 다시 시도해주세요.',
          );
        case ApiErrorKind.network:
          return const UiError(
            code: UiErrorCode.network,
            message: '네트워크 연결을 확인해주세요.',
          );
        case ApiErrorKind.timeout:
          return const UiError(
            code: UiErrorCode.timeout,
            message: '요청 시간이 초과되었습니다. 다시 시도해주세요.',
          );
        case ApiErrorKind.client:
          return UiError(code: UiErrorCode.client, message: error.message);
        case ApiErrorKind.unknown:
          return UiError(code: UiErrorCode.unknown, message: error.message);
      }
    }

    return UiError(code: UiErrorCode.unknown, message: error.toString());
  }
}
