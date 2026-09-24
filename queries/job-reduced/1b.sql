WITH ct_0 AS MATERIALIZED (SELECT id FROM company_type AS ct WHERE (ct.kind = 'production companies')),
it_0 AS MATERIALIZED (SELECT id FROM info_type AS it WHERE (it.info = 'bottom 10 rank')),
mc_0 AS MATERIALIZED (SELECT company_type_id, movie_id, note FROM movie_companies AS mc WHERE (mc.note NOT LIKE '%(as Metro-Goldwyn-Mayer Pictures)%')),
mi_idx_0 AS MATERIALIZED (SELECT info_type_id, movie_id FROM movie_info_idx AS mi_idx),
t_0 AS MATERIALIZED (SELECT id, production_year, title FROM title AS t WHERE (t.production_year BETWEEN 2005 AND 2010)),
mc_1 AS MATERIALIZED (SELECT * FROM mc_0 AS mc_t WHERE EXISTS (SELECT 1 FROM ct_0 AS ct_s WHERE ct_s.id = mc_t.company_type_id)),
mi_idx_1 AS MATERIALIZED (SELECT * FROM mi_idx_0 AS mi_idx_t WHERE EXISTS (SELECT 1 FROM it_0 AS it_s WHERE it_s.id = mi_idx_t.info_type_id)),
mi_idx_2 AS MATERIALIZED (SELECT * FROM mi_idx_1 AS mi_idx_t WHERE EXISTS (SELECT 1 FROM mc_1 AS mc_s WHERE mc_s.movie_id = mi_idx_t.movie_id)),
t_1 AS MATERIALIZED (SELECT * FROM t_0 AS t_t WHERE EXISTS (SELECT 1 FROM mi_idx_2 AS mi_idx_s WHERE mi_idx_s.movie_id = t_t.id)),
mi_idx_3 AS MATERIALIZED (SELECT * FROM mi_idx_2 AS mi_idx_t WHERE EXISTS (SELECT 1 FROM t_1 AS t_s WHERE t_s.id = mi_idx_t.movie_id)),
mc_2 AS MATERIALIZED (SELECT * FROM mc_1 AS mc_t WHERE EXISTS (SELECT 1 FROM mi_idx_3 AS mi_idx_s WHERE mi_idx_s.movie_id = mc_t.movie_id)),
it_1 AS MATERIALIZED (SELECT * FROM it_0 AS it_t WHERE EXISTS (SELECT 1 FROM mi_idx_3 AS mi_idx_s WHERE mi_idx_s.info_type_id = it_t.id)),
ct_1 AS MATERIALIZED (SELECT * FROM ct_0 AS ct_t WHERE EXISTS (SELECT 1 FROM mc_2 AS mc_s WHERE mc_s.company_type_id = ct_t.id))
SELECT (SELECT MIN(note) FROM mc_2) AS production_note, (SELECT MIN(title) FROM t_1) AS movie_title, (SELECT MIN(production_year) FROM t_1) AS movie_year
