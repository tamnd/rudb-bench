WITH k_0 AS MATERIALIZED (SELECT id FROM keyword AS k WHERE (k.keyword LIKE '%sequel%')),
mi_0 AS MATERIALIZED (SELECT movie_id FROM movie_info AS mi WHERE (mi.info IN ('Bulgaria'))),
mk_0 AS MATERIALIZED (SELECT keyword_id, movie_id FROM movie_keyword AS mk),
t_0 AS MATERIALIZED (SELECT id, title FROM title AS t WHERE (t.production_year > 2010)),
mk_1 AS MATERIALIZED (SELECT * FROM mk_0 AS mk_t WHERE EXISTS (SELECT 1 FROM k_0 AS k_s WHERE k_s.id = mk_t.keyword_id)),
mk_2 AS MATERIALIZED (SELECT * FROM mk_1 AS mk_t WHERE EXISTS (SELECT 1 FROM mi_0 AS mi_s WHERE mi_s.movie_id = mk_t.movie_id)),
t_1 AS MATERIALIZED (SELECT * FROM t_0 AS t_t WHERE EXISTS (SELECT 1 FROM mk_2 AS mk_s WHERE mk_s.movie_id = t_t.id)),
mk_3 AS MATERIALIZED (SELECT * FROM mk_2 AS mk_t WHERE EXISTS (SELECT 1 FROM t_1 AS t_s WHERE t_s.id = mk_t.movie_id)),
mi_1 AS MATERIALIZED (SELECT * FROM mi_0 AS mi_t WHERE EXISTS (SELECT 1 FROM mk_3 AS mk_s WHERE mk_s.movie_id = mi_t.movie_id)),
k_1 AS MATERIALIZED (SELECT * FROM k_0 AS k_t WHERE EXISTS (SELECT 1 FROM mk_3 AS mk_s WHERE mk_s.keyword_id = k_t.id))
SELECT (SELECT MIN(title) FROM t_1) AS movie_title
