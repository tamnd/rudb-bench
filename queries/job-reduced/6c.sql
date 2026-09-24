WITH ci_0 AS MATERIALIZED (SELECT movie_id, person_id FROM cast_info AS ci),
k_0 AS MATERIALIZED (SELECT id, keyword FROM keyword AS k WHERE (k.keyword = 'marvel-cinematic-universe')),
mk_0 AS MATERIALIZED (SELECT keyword_id, movie_id FROM movie_keyword AS mk),
n_0 AS MATERIALIZED (SELECT id, name FROM name AS n WHERE (n.name LIKE '%Downey%Robert%')),
t_0 AS MATERIALIZED (SELECT id, title FROM title AS t WHERE (t.production_year > 2014)),
mk_1 AS MATERIALIZED (SELECT * FROM mk_0 AS mk_t WHERE EXISTS (SELECT 1 FROM k_0 AS k_s WHERE k_s.id = mk_t.keyword_id)),
ci_1 AS MATERIALIZED (SELECT * FROM ci_0 AS ci_t WHERE EXISTS (SELECT 1 FROM mk_1 AS mk_s WHERE mk_s.movie_id = ci_t.movie_id)),
ci_2 AS MATERIALIZED (SELECT * FROM ci_1 AS ci_t WHERE EXISTS (SELECT 1 FROM n_0 AS n_s WHERE n_s.id = ci_t.person_id)),
t_1 AS MATERIALIZED (SELECT * FROM t_0 AS t_t WHERE EXISTS (SELECT 1 FROM ci_2 AS ci_s WHERE ci_s.movie_id = t_t.id)),
ci_3 AS MATERIALIZED (SELECT * FROM ci_2 AS ci_t WHERE EXISTS (SELECT 1 FROM t_1 AS t_s WHERE t_s.id = ci_t.movie_id)),
n_1 AS MATERIALIZED (SELECT * FROM n_0 AS n_t WHERE EXISTS (SELECT 1 FROM ci_3 AS ci_s WHERE ci_s.person_id = n_t.id)),
mk_2 AS MATERIALIZED (SELECT * FROM mk_1 AS mk_t WHERE EXISTS (SELECT 1 FROM ci_3 AS ci_s WHERE ci_s.movie_id = mk_t.movie_id)),
k_1 AS MATERIALIZED (SELECT * FROM k_0 AS k_t WHERE EXISTS (SELECT 1 FROM mk_2 AS mk_s WHERE mk_s.keyword_id = k_t.id))
SELECT (SELECT MIN(keyword) FROM k_1) AS movie_keyword, (SELECT MIN(name) FROM n_1) AS actor_name, (SELECT MIN(title) FROM t_1) AS marvel_movie
