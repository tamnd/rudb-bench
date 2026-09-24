WITH an_0 AS MATERIALIZED (SELECT name, person_id FROM aka_name AS an),
ci_0 AS MATERIALIZED (SELECT movie_id, person_id, role_id FROM cast_info AS ci WHERE (ci.note ='(voice: English version)')),
cn_0 AS MATERIALIZED (SELECT id FROM company_name AS cn WHERE (cn.country_code ='[jp]')),
mc_0 AS MATERIALIZED (SELECT company_id, movie_id FROM movie_companies AS mc WHERE (mc.note LIKE '%(Japan)%') AND (mc.note NOT LIKE '%(USA)%') AND ((mc.note LIKE '%(2006)%'
       OR mc.note LIKE '%(2007)%'))),
n_0 AS MATERIALIZED (SELECT id FROM name AS n WHERE (n.name LIKE '%Yo%') AND (n.name NOT LIKE '%Yu%')),
rt_0 AS MATERIALIZED (SELECT id FROM role_type AS rt WHERE (rt.role ='actress')),
t_0 AS MATERIALIZED (SELECT id, title FROM title AS t WHERE (t.production_year BETWEEN 2006 AND 2007) AND ((t.title LIKE 'One Piece%'
       OR t.title LIKE 'Dragon Ball Z%'))),
ci_1 AS MATERIALIZED (SELECT * FROM ci_0 AS ci_t WHERE EXISTS (SELECT 1 FROM an_0 AS an_s WHERE an_s.person_id = ci_t.person_id)),
mc_1 AS MATERIALIZED (SELECT * FROM mc_0 AS mc_t WHERE EXISTS (SELECT 1 FROM cn_0 AS cn_s WHERE cn_s.id = mc_t.company_id)),
ci_2 AS MATERIALIZED (SELECT * FROM ci_1 AS ci_t WHERE EXISTS (SELECT 1 FROM mc_1 AS mc_s WHERE mc_s.movie_id = ci_t.movie_id)),
ci_3 AS MATERIALIZED (SELECT * FROM ci_2 AS ci_t WHERE EXISTS (SELECT 1 FROM n_0 AS n_s WHERE n_s.id = ci_t.person_id)),
ci_4 AS MATERIALIZED (SELECT * FROM ci_3 AS ci_t WHERE EXISTS (SELECT 1 FROM rt_0 AS rt_s WHERE rt_s.id = ci_t.role_id)),
t_1 AS MATERIALIZED (SELECT * FROM t_0 AS t_t WHERE EXISTS (SELECT 1 FROM ci_4 AS ci_s WHERE ci_s.movie_id = t_t.id)),
ci_5 AS MATERIALIZED (SELECT * FROM ci_4 AS ci_t WHERE EXISTS (SELECT 1 FROM t_1 AS t_s WHERE t_s.id = ci_t.movie_id)),
rt_1 AS MATERIALIZED (SELECT * FROM rt_0 AS rt_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.role_id = rt_t.id)),
n_1 AS MATERIALIZED (SELECT * FROM n_0 AS n_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.person_id = n_t.id)),
mc_2 AS MATERIALIZED (SELECT * FROM mc_1 AS mc_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.movie_id = mc_t.movie_id)),
cn_1 AS MATERIALIZED (SELECT * FROM cn_0 AS cn_t WHERE EXISTS (SELECT 1 FROM mc_2 AS mc_s WHERE mc_s.company_id = cn_t.id)),
an_1 AS MATERIALIZED (SELECT * FROM an_0 AS an_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.person_id = an_t.person_id))
SELECT (SELECT MIN(name) FROM an_1) AS acress_pseudonym, (SELECT MIN(title) FROM t_1) AS japanese_anime_movie
