+++
title = "Materialized View"
date = "2026-07-12"
aliases = ["posts/2026-07-12-materialized-view"]
description = "집계, 통계 등 거대한 연산에 유리한 MV 알아보기"

[taxonomies]
tags = ["database"]
+++

흔히 온라인 게임 랭킹에서 볼 수 있는 이 순위표, 다들 한번 쯤 본 적이 있을 것이다.

![ranking-image](../../images/valorant-ranking.jpg)
> 게임 '발로란트'의 랭킹 순위표

근래에 재미있는 릴스를 봤는데 이런 랭킹을 구현하려면 `Redis` 같은 캐시 서버를 써서 결과를 '캐시'하라는 내용이다. (대충 브레인랏 형식으로 설명하는 릴스) 물론 틀린 말은 아닌데 이러면 사실 복잡도를 의존하게 된다. 뿐만 아니라 추후 관리 포인트도 늘 수 있어서.. 별로 좋은 생각은 아닌 것 같다.

관리 포인트를 줄여주면서 데이터베이스에서 기본적으로 제공해주는 기능중에 **Materialized View** 라는 유용한 기능이 있다. 직역하자면 **구체화된 뷰**라는 뜻인데, 쿼리 결과를 미리 계산해서 테이블 형태로 저장한다. 즉, 랭킹 뿐만 아니라 어떤 큰 쿼리 집계(e.g 대쉬보드, 사용량 분석 등등..)에 맞는 기능이다.

이게 한번 계산한 테이블은 영구적이라, 특정 주기나 수동으로 Resync가 필요하다. 그러니까 어떤 트리거가 없으면 원본 테이블에 변화가 있어도 계산된 값은 바뀌지 않는다.

대표적으로 **MV**를 지원하는 데이터베이스 엔진은 `Oracle`이랑 `PostgreSQL`이 있다. 해당 글에서는 `PostgreSQL` 기준으로 설명한다.

직접 한번 만들어보자.

---

```bash
developer@arch-b9 ~ % docker run --name mv-demo -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres:18 
b5d221c60831c7017f01500decff8f172f3d0c9322330a592256485a31296988
developer@arch-b9 ~ % 
```

이렇게 18버전 서버를 하나 띄우고, 간단한 스코어 테이블을 만들고 값들을 채워보자.
```bash
developer@arch-b9 ~ % docker exec -it mv-demo psql -U postgres
psql (18.1 (Debian 18.1-1.pgdg13+2))
Type "help" for help.

postgres=# CREATE TABLE game_scores (
    user_id INT PRIMARY KEY,
    username TEXT NOT NULL,
    score INT NOT NULL
);
CREATE TABLE

postgres=# INSERT INTO game_scores VALUES
    (1,'alice',1500),(2,'bob',2300),(3,'carol',1800),(4,'dave',900),(5,'erin',2100);
INSERT 0 5
```

그리고 이제 저 데이터들을 가지고 MV를 만들어보겠다. (실제 프로덕션 테이블엔 컬럼도 훨씬 많고 복잡할 것이다.)

```bash
postgres=# CREATE MATERIALIZED VIEW ranking AS
SELECT ROW_NUMBER() OVER (ORDER BY score DESC) AS rank, user_id, username, score
FROM game_scores;
```

이러면 `ranking`이란 이름을 가진 뷰가 완성이 되는데, 한번 찍어보자.

```bash
postgres=# SELECT * FROM ranking ORDER BY rank;
 rank | user_id | username | score 
------+---------+----------+-------
    1 |       2 | bob      |  2300
    2 |       5 | erin     |  2100
    3 |       3 | carol    |  1800
    4 |       1 | alice    |  1500
    5 |       4 | dave     |   900
(5 rows)
```

이렇게 잘 나오는 것을 볼 수 있다. 원본을 한번 바꾸고 다시 조회해보자.

```bash

postgres=# UPDATE game_scores SET score = 9999 WHERE user_id = 4;
UPDATE 1

postgres=# SELECT * FROM ranking ORDER BY rank;
 rank | user_id | username | score 
------+---------+----------+-------
    1 |       2 | bob      |  2300
    2 |       5 | erin     |  2100
    3 |       3 | carol    |  1800
    4 |       1 | alice    |  1500
    5 |       4 | dave     |   900
(5 rows)
```

보면 옛날 랭킹이 그대로 나오는 것을 확인할 수 있다. 이제 refresh를 해주면..

```bash
postgres=# REFRESH MATERIALIZED VIEW ranking;

postgres=# SELECT * FROM ranking;
 rank | user_id | username | score 
------+---------+----------+-------
    1 |       4 | dave     |  9999
    2 |       2 | bob      |  2300
    3 |       5 | erin     |  2100
    4 |       3 | carol    |  1800
    5 |       1 | alice    |  1500
(5 rows)
```

이렇게 갱신하면 제대로 나오는 걸 볼 수 있다.

문제는, 리프레시할 때 뷰를 재계산하기 때문에 해당 MV는 락이 걸려 조회할 수 없다. 이를 해결하기 위해 **CONCURRENTLY** 라는 옵션이 있는데 이건 새로 계산하고 거기서 **diff** 계산을 또 한다. 그리고 바뀐 행만 반영한다.

아무래도 중간에 바뀐 걸 비교하고 스왑하니까 좀 더 비용적으로 무거운데, 읽기가 절대 막히면 안되는 상황에서 쓸만하다. 다만, 저거 하려면 유니크 인덱스가 필요하다.

대강 알아봤는데 실무에선 pgcron 같은거 묶어서 쓸 것 같다. 좋은 기능이다.
