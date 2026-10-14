# 12. 내 데이터베이스(MySQL 말을 하는 MariaDB)

> 영어 원문(전체 안내서): [../12-your-database.md](../12-your-database.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래 링크(영어)에 있어요.*

- 정원의 데이터베이스 이름은 `juns_garden_db`예요. `garden-db` 컨테이너에서 실행되고, 네트워크에서는 절대 들어올 수 없어요.
- 식물은 `data_source` 표에 있어요.
- 브라우저에서 Adminer로 안을 보기: 메뉴 `M`, 그다음 `8`, 그리고 <http://localhost:8082>를 여세요. 다 보면 `bash garden.sh dbweb-stop`으로 멈추세요.
- 또는 SQL을 입력하기: `bash garden.sh db`.
- 데이터를 직접 바꿀 때마다 먼저 백업하세요(메뉴 `6`, 그다음 `1`). 실수했나요? 그 백업을 복원하세요(메뉴 `7`).
- `UPDATE`와 `DELETE`에는 항상 `WHERE`를 쓰세요. 없으면 모든 줄이 바뀌어요.
- 대부분의 변경은 식물 관리(`/streamlit/`)에서 하는 것이 더 쉽고 안전해요.

---

전체 내용은 영어 안내서에 있어요: [../12-your-database.md](../12-your-database.md)
