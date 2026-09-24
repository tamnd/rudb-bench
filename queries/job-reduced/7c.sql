WITH an_0 AS MATERIALIZED (SELECT person_id FROM aka_name AS an WHERE (an.name IS NOT NULL) AND ((an.name LIKE '%a%'
       OR an.name LIKE 'A%'))),
ci_0 AS MATERIALIZED (SELECT movie_id, person_id FROM cast_info AS ci),
it_0 AS MATERIALIZED (SELECT id FROM info_type AS it WHERE (it.info ='mini biography')),
lt_0 AS MATERIALIZED (SELECT id FROM link_type AS lt WHERE (lt.link IN ('references',
                  'referenced in',
                  'features',
                  'featured in'))),
ml_0 AS MATERIALIZED (SELECT link_type_id, linked_movie_id FROM movie_link AS ml),
n_0 AS MATERIALIZED (SELECT id, name FROM name AS n WHERE (n.name_pcode_cf BETWEEN 'A' AND 'F') AND ((n.gender='m'
       OR (n.gender = 'f'
           AND n.name LIKE 'A%')))),
pi_0 AS MATERIALIZED (SELECT info, info_type_id, person_id FROM person_info AS pi WHERE (pi.note IS NOT NULL)),
t_0 AS MATERIALIZED (SELECT id FROM title AS t WHERE (t.production_year BETWEEN 1980 AND 2010)),
ci_1 AS MATERIALIZED (SELECT * FROM ci_0 AS ci_t WHERE EXISTS (SELECT 1 FROM an_0 AS an_s WHERE an_s.person_id = ci_t.person_id)),
pi_1 AS MATERIALIZED (SELECT * FROM pi_0 AS pi_t WHERE EXISTS (SELECT 1 FROM it_0 AS it_s WHERE it_s.id = pi_t.info_type_id)),
ml_1 AS MATERIALIZED (SELECT * FROM ml_0 AS ml_t WHERE EXISTS (SELECT 1 FROM lt_0 AS lt_s WHERE lt_s.id = ml_t.link_type_id)),
ci_2 AS MATERIALIZED (SELECT * FROM ci_1 AS ci_t WHERE EXISTS (SELECT 1 FROM ml_1 AS ml_s WHERE ml_s.linked_movie_id = ci_t.movie_id)),
ci_3 AS MATERIALIZED (SELECT * FROM ci_2 AS ci_t WHERE EXISTS (SELECT 1 FROM n_0 AS n_s WHERE n_s.id = ci_t.person_id)),
ci_4 AS MATERIALIZED (SELECT * FROM ci_3 AS ci_t WHERE EXISTS (SELECT 1 FROM pi_1 AS pi_s WHERE pi_s.person_id = ci_t.person_id)),
t_1 AS MATERIALIZED (SELECT * FROM t_0 AS t_t WHERE EXISTS (SELECT 1 FROM ci_4 AS ci_s WHERE ci_s.movie_id = t_t.id)),
ci_5 AS MATERIALIZED (SELECT * FROM ci_4 AS ci_t WHERE EXISTS (SELECT 1 FROM t_1 AS t_s WHERE t_s.id = ci_t.movie_id)),
pi_2 AS MATERIALIZED (SELECT * FROM pi_1 AS pi_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.person_id = pi_t.person_id)),
n_1 AS MATERIALIZED (SELECT * FROM n_0 AS n_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.person_id = n_t.id)),
ml_2 AS MATERIALIZED (SELECT * FROM ml_1 AS ml_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.movie_id = ml_t.linked_movie_id)),
lt_1 AS MATERIALIZED (SELECT * FROM lt_0 AS lt_t WHERE EXISTS (SELECT 1 FROM ml_2 AS ml_s WHERE ml_s.link_type_id = lt_t.id)),
it_1 AS MATERIALIZED (SELECT * FROM it_0 AS it_t WHERE EXISTS (SELECT 1 FROM pi_2 AS pi_s WHERE pi_s.info_type_id = it_t.id)),
an_1 AS MATERIALIZED (SELECT * FROM an_0 AS an_t WHERE EXISTS (SELECT 1 FROM ci_5 AS ci_s WHERE ci_s.person_id = an_t.person_id))
SELECT (SELECT MIN(name) FROM n_1) AS cast_member_name, (SELECT MIN(info) FROM pi_2) AS cast_member_info
