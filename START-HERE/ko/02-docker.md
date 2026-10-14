# 02. Docker 설치하기

> **이것을 손으로 할 필요는 없어요: 설치(메뉴 1)가 Docker를 대신 설치해 줘요. 이 안내서는
> 설명만 해요.**

> 영어 원문(전체 안내서): [../02-docker.md](../02-docker.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래 링크(영어)에 있어요.*

- 이 안내서는 Ubuntu 노트북에 Docker를 설치해요. Docker는 프로그램을 컨테이너(container)라는 밀봉된 상자 안에서 실행해요.
- 30분 걸려요. 무료예요.
- 1~3단계는 Docker의 공식 페이지를 따라요: 예전 조각을 지우고, Docker의 소프트웨어 출처를 추가하고, Docker와 Compose를 설치해요. Docker가 단계를 바꾸면 그쪽 페이지를 믿으세요.
- 4단계: `sudo usermod -aG docker $USER`, 그다음 **로그아웃했다가 다시 로그인하세요**. 잊어버리기 쉬운 단계예요.
- 테스트하기: `docker run --rm hello-world`. "Hello from Docker!"가 보여야 해요.
- `permission denied ... docker.sock`이 나오면, 4단계 뒤에 로그아웃했다가 다시 로그인하지 않은 거예요.

---

전체 내용은 영어 안내서에 있어요: [../02-docker.md](../02-docker.md)
