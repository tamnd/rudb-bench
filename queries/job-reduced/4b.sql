WITH it_0 AS MATERIALIZED (SELECT id FROM info_type AS it WHERE (it.info ='rating')),
k_0 AS MATERIALIZED (SELECT id FROM keyword AS k WHERE (k.keyword LIKE '%sequel%')),
mi_idx_0 AS MATERIALIZED (SELECT info, info_type_id, movie_id FROM movie_info_idx AS mi_idx WHERE (mi_idx.info > '9.0')),
mk_0 AS MATERIALIZED (SELECT keyword_id, movie_id FROM movie_keyword AS mk),
t_0 AS MATERIALIZED (SELECT id, title FROM title AS t WHERE (t.production_year > 2010)),
mi_idx_1 AS MATERIALIZED (SELECT * FROM mi_idx_0 AS mi_idx_t WHERE EXISTS (SELECT 1 FROM it_0 AS it_s WHERE it_s.id = mi_idx_t.info_type_id)),
mk_1 AS MATERIALIZED (SELECT * FROM mk_0 AS mk_t WHERE EXISTS (SELECT 1 FROM k_0 AS k_s WHERE k_s.id = mk_t.keyword_id)),
mk_2 AS MATERIALIZED (SELECT * FROM mk_1 AS mk_t WHERE EXISTS (SELECT 1 FROM mi_idx_1 AS mi_idx_s WHERE mi_idx_s.movie_id = mk_t.movie_id)),
t_1 AS MATERIALIZED (SELECT * FROM t_0 AS t_t WHERE EXISTS (SELECT 1 FROM mk_2 AS mk_s WHERE mk_s.movie_id = t_t.id)),
mk_3 AS MATERIALIZED (SELECT * FROM mk_2 AS mk_t WHERE EXISTS (SELECT 1 FROM t_1 AS t_s WHERE t_s.id = mk_t.movie_id)),
mi_idx_2 AS MATERIALIZED (SELECT * FROM mi_idx_1 AS mi_idx_t WHERE EXISTS (SELECT 1 FROM mk_3 AS mk_s WHERE mk_s.movie_id = mi_idx_t.movie_id)),
k_1 AS MATERIALIZED (SELECT * FROM k_0 AS k_t WHERE EXISTS (SELECT 1 FROM mk_3 AS mk_s WHERE mk_s.keyword_id = k_t.id)),
it_1 AS MATERIALIZED (SELECT * FROM it_0 AS it_t WHERE EXISTS (SELECT 1 FROM mi_idx_2 AS mi_idx_s WHERE mi_idx_s.info_type_id = it_t.id))
SELECT (SELECT MIN(info) FROM mi_idx_2) AS rating, (SELECT MIN(title) FROM t_1) AS movie_title
