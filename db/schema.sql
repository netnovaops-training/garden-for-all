-- Structure of the garden database (juns_garden_db), no data.
-- Generated 2026-10-05 from the live database. Use it to set up a new machine:
--   mariadb -u root -p < db/schema.sql
-- Data is NOT included; see docs/rust-port/README.md for backups.

/*M!999999\- enable the sandbox mode */ 

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*M!100616 SET @OLD_NOTE_VERBOSITY=@@NOTE_VERBOSITY, NOTE_VERBOSITY=0 */;

CREATE DATABASE /*!32312 IF NOT EXISTS*/ `juns_garden_db` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_uca1400_ai_ci */;

USE `juns_garden_db`;
DROP TABLE IF EXISTS `Annuals`;
/*!50001 DROP VIEW IF EXISTS `Annuals`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `Annuals` AS SELECT
 NULL AS `id`,
 NULL AS `family`,
 NULL AS `picture`,
 NULL AS `plant_name`,
 NULL AS `company`,
 NULL AS `local_store`,
 NULL AS `qty`,
 NULL AS `died`,
 NULL AS `new_total`,
 NULL AS `year_planted`,
 NULL AS `split_amt`,
 NULL AS `sun_exposure`,
 NULL AS `cleanup_notes`,
 NULL AS `care_notes`,
 NULL AS `cuttings`,
 NULL AS `fertilizer`,
 NULL AS `size_spacing`,
 NULL AS `new_name`,
 NULL AS `flower_bed`,
 NULL AS `additional_pictures`,
 NULL AS `maintenance_notes`,
 NULL AS `attracts`,
 NULL AS `zones`,
 NULL AS `water_needs`,
 NULL AS `deer_resistance`,
 NULL AS `season`,
 NULL AS `month_start`,
 NULL AS `month_end`,
 NULL AS `planting_month_start`,
 NULL AS `planting_month_end`,
 NULL AS `spacing`,
 NULL AS `admin_todo`,
 NULL AS `botanical_name`,
 NULL AS `soil_ph`,
 NULL AS `soil_type`,
 NULL AS `toxicity`,
 NULL AS `bloom_color`,
 NULL AS `foliage_color`,
 NULL AS `disease_susceptibility`,
 NULL AS `propagation_method`,
 NULL AS `species`,
 NULL AS `size`,
 NULL AS `grow_months`,
 NULL AS `bloom_months`,
 NULL AS `average_cost`,
 NULL AS `extra_details`,
 NULL AS `image_search_term`,
 NULL AS `plant_type`,
 NULL AS `image_verified`,
 NULL AS `stem_color`,
 NULL AS `genus`,
 NULL AS `leaf_shape`,
 NULL AS `leaf_type`,
 NULL AS `leaf_margin`,
 NULL AS `flower_symmetry`,
 NULL AS `flower_shape`,
 NULL AS `fruit_type`,
 NULL AS `fruit_shape`,
 NULL AS `fruit_color`,
 NULL AS `fruit_size`,
 NULL AS `fruit_edibility`,
 NULL AS `invasive_local`,
 NULL AS `native_status`,
 NULL AS `growth_rate`,
 NULL AS `planting_depth`,
 NULL AS `chilling_hours`,
 NULL AS `companion_plants`,
 NULL AS `approved_jason`,
 NULL AS `approved_jun`,
 NULL AS `needs_review`,
 NULL AS `garden_beds`,
 NULL AS `aromatics`,
 NULL AS `flower_size`,
 NULL AS `is_evergreen`,
 NULL AS `quantity`,
 NULL AS `year_added`,
 NULL AS `plant_length`,
 NULL AS `plant_width`,
 NULL AS `life_expectancy`,
 NULL AS `reproduction_type`,
 NULL AS `root_system`,
 NULL AS `root_morphology`,
 NULL AS `root_function`,
 NULL AS `grow_season`,
 NULL AS `died_year`,
 NULL AS `tag_done`,
 NULL AS `photo_position`,
 NULL AS `photo_flip`,
 NULL AS `image_approved`,
 NULL AS `qr_code`,
 NULL AS `barcode`,
 NULL AS `barcode_image`,
 NULL AS `plant_length_min_in`,
 NULL AS `plant_length_max_in`,
 NULL AS `plant_width_min_in`,
 NULL AS `plant_width_max_in`,
 NULL AS `spacing_min_in`,
 NULL AS `spacing_max_in` */;
SET character_set_client = @saved_cs_client;
DROP TABLE IF EXISTS `Perennials`;
/*!50001 DROP VIEW IF EXISTS `Perennials`*/;
SET @saved_cs_client     = @@character_set_client;
SET character_set_client = utf8mb4;
/*!50001 CREATE VIEW `Perennials` AS SELECT
 NULL AS `id`,
 NULL AS `family`,
 NULL AS `picture`,
 NULL AS `plant_name`,
 NULL AS `company`,
 NULL AS `local_store`,
 NULL AS `qty`,
 NULL AS `died`,
 NULL AS `new_total`,
 NULL AS `year_planted`,
 NULL AS `split_amt`,
 NULL AS `sun_exposure`,
 NULL AS `cleanup_notes`,
 NULL AS `care_notes`,
 NULL AS `cuttings`,
 NULL AS `fertilizer`,
 NULL AS `size_spacing`,
 NULL AS `new_name`,
 NULL AS `flower_bed`,
 NULL AS `additional_pictures`,
 NULL AS `maintenance_notes`,
 NULL AS `attracts`,
 NULL AS `zones`,
 NULL AS `water_needs`,
 NULL AS `deer_resistance`,
 NULL AS `season`,
 NULL AS `month_start`,
 NULL AS `month_end`,
 NULL AS `planting_month_start`,
 NULL AS `planting_month_end`,
 NULL AS `spacing`,
 NULL AS `admin_todo`,
 NULL AS `botanical_name`,
 NULL AS `soil_ph`,
 NULL AS `soil_type`,
 NULL AS `toxicity`,
 NULL AS `bloom_color`,
 NULL AS `foliage_color`,
 NULL AS `disease_susceptibility`,
 NULL AS `propagation_method`,
 NULL AS `species`,
 NULL AS `size`,
 NULL AS `grow_months`,
 NULL AS `bloom_months`,
 NULL AS `average_cost`,
 NULL AS `extra_details`,
 NULL AS `image_search_term`,
 NULL AS `plant_type`,
 NULL AS `image_verified`,
 NULL AS `stem_color`,
 NULL AS `genus`,
 NULL AS `leaf_shape`,
 NULL AS `leaf_type`,
 NULL AS `leaf_margin`,
 NULL AS `flower_symmetry`,
 NULL AS `flower_shape`,
 NULL AS `fruit_type`,
 NULL AS `fruit_shape`,
 NULL AS `fruit_color`,
 NULL AS `fruit_size`,
 NULL AS `fruit_edibility`,
 NULL AS `invasive_local`,
 NULL AS `native_status`,
 NULL AS `growth_rate`,
 NULL AS `planting_depth`,
 NULL AS `chilling_hours`,
 NULL AS `companion_plants`,
 NULL AS `approved_jason`,
 NULL AS `approved_jun`,
 NULL AS `needs_review`,
 NULL AS `garden_beds`,
 NULL AS `aromatics`,
 NULL AS `flower_size`,
 NULL AS `is_evergreen`,
 NULL AS `quantity`,
 NULL AS `year_added`,
 NULL AS `plant_length`,
 NULL AS `plant_width`,
 NULL AS `life_expectancy`,
 NULL AS `reproduction_type`,
 NULL AS `root_system`,
 NULL AS `root_morphology`,
 NULL AS `root_function`,
 NULL AS `grow_season`,
 NULL AS `died_year`,
 NULL AS `tag_done`,
 NULL AS `photo_position`,
 NULL AS `photo_flip`,
 NULL AS `image_approved`,
 NULL AS `qr_code`,
 NULL AS `barcode`,
 NULL AS `barcode_image`,
 NULL AS `plant_length_min_in`,
 NULL AS `plant_length_max_in`,
 NULL AS `plant_width_min_in`,
 NULL AS `plant_width_max_in`,
 NULL AS `spacing_min_in`,
 NULL AS `spacing_max_in` */;
SET character_set_client = @saved_cs_client;
DROP TABLE IF EXISTS `_species_snapshot`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `_species_snapshot` (
  `id` int(11) NOT NULL DEFAULT 0,
  `species` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `data_source`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `data_source` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `family` varchar(255) DEFAULT NULL,
  `picture` text DEFAULT NULL,
  `plant_name` varchar(255) DEFAULT NULL,
  `company` varchar(100) DEFAULT NULL,
  `local_store` varchar(255) DEFAULT NULL,
  `qty` int(11) DEFAULT 0,
  `died` int(11) DEFAULT 0,
  `new_total` int(11) DEFAULT 0,
  `year_planted` varchar(50) DEFAULT NULL,
  `split_amt` varchar(50) DEFAULT NULL,
  `sun_exposure` varchar(100) DEFAULT NULL,
  `cleanup_notes` text DEFAULT NULL,
  `care_notes` text DEFAULT NULL,
  `cuttings` varchar(255) DEFAULT NULL,
  `fertilizer` varchar(100) DEFAULT NULL,
  `size_spacing` varchar(100) DEFAULT NULL,
  `new_name` varchar(255) DEFAULT NULL,
  `flower_bed` varchar(100) DEFAULT NULL,
  `additional_pictures` text DEFAULT NULL,
  `maintenance_notes` text DEFAULT NULL,
  `attracts` text DEFAULT NULL,
  `zones` varchar(255) DEFAULT NULL,
  `water_needs` varchar(255) DEFAULT NULL,
  `deer_resistance` varchar(50) DEFAULT NULL,
  `season` varchar(100) DEFAULT NULL,
  `month_start` varchar(20) DEFAULT NULL,
  `month_end` varchar(20) DEFAULT NULL,
  `planting_month_start` varchar(20) DEFAULT NULL,
  `planting_month_end` varchar(20) DEFAULT NULL,
  `spacing` varchar(50) DEFAULT NULL,
  `admin_todo` varchar(255) DEFAULT NULL,
  `botanical_name` varchar(150) DEFAULT NULL,
  `soil_ph` varchar(255) DEFAULT NULL,
  `soil_type` varchar(255) DEFAULT NULL,
  `toxicity` varchar(255) DEFAULT NULL,
  `bloom_color` varchar(255) DEFAULT NULL,
  `foliage_color` varchar(255) DEFAULT NULL,
  `disease_susceptibility` text DEFAULT NULL,
  `propagation_method` varchar(255) DEFAULT NULL,
  `species` varchar(255) DEFAULT NULL,
  `size` varchar(255) DEFAULT NULL,
  `grow_months` varchar(255) DEFAULT NULL,
  `bloom_months` varchar(255) DEFAULT NULL,
  `average_cost` decimal(10,2) DEFAULT NULL,
  `extra_details` text DEFAULT NULL,
  `image_search_term` varchar(255) DEFAULT NULL,
  `plant_type` varchar(50) DEFAULT NULL,
  `image_verified` tinyint(1) DEFAULT 0,
  `stem_color` varchar(255) DEFAULT NULL,
  `genus` varchar(255) DEFAULT NULL,
  `leaf_shape` varchar(255) DEFAULT NULL,
  `leaf_type` varchar(255) DEFAULT NULL,
  `leaf_margin` varchar(255) DEFAULT NULL,
  `flower_symmetry` varchar(255) DEFAULT NULL,
  `flower_shape` varchar(255) DEFAULT NULL,
  `fruit_type` varchar(255) DEFAULT NULL,
  `fruit_shape` varchar(255) DEFAULT NULL,
  `fruit_color` varchar(255) DEFAULT NULL,
  `fruit_size` varchar(255) DEFAULT NULL,
  `fruit_edibility` varchar(255) DEFAULT NULL,
  `invasive_local` varchar(50) DEFAULT NULL,
  `native_status` varchar(50) DEFAULT NULL,
  `growth_rate` varchar(50) DEFAULT NULL,
  `planting_depth` varchar(255) DEFAULT NULL,
  `chilling_hours` varchar(255) DEFAULT NULL,
  `companion_plants` text DEFAULT NULL,
  `approved_jason` tinyint(1) DEFAULT 0,
  `approved_jun` tinyint(1) DEFAULT 0,
  `needs_review` tinyint(1) DEFAULT 0,
  `garden_beds` varchar(500) DEFAULT NULL,
  `aromatics` varchar(255) DEFAULT NULL,
  `flower_size` varchar(255) DEFAULT NULL,
  `is_evergreen` varchar(50) DEFAULT NULL,
  `quantity` int(11) DEFAULT 1,
  `year_added` varchar(255) DEFAULT NULL,
  `plant_length` varchar(255) DEFAULT NULL,
  `plant_width` varchar(255) DEFAULT NULL,
  `life_expectancy` varchar(255) DEFAULT NULL,
  `reproduction_type` varchar(255) DEFAULT NULL,
  `root_system` varchar(255) DEFAULT NULL,
  `root_morphology` varchar(255) DEFAULT NULL,
  `root_function` varchar(255) DEFAULT NULL,
  `grow_season` varchar(60) DEFAULT NULL,
  `died_year` smallint(5) unsigned DEFAULT NULL COMMENT 'Year the plant died (NULL = still alive)',
  `tag_done` tinyint(1) NOT NULL DEFAULT 0,
  `photo_position` varchar(30) DEFAULT '50% 50%',
  `photo_flip` tinyint(1) DEFAULT 0,
  `image_approved` tinyint(1) DEFAULT 0,
  `qr_code` mediumblob DEFAULT NULL,
  `barcode` varchar(20) DEFAULT NULL,
  `barcode_image` mediumblob DEFAULT NULL,
  `plant_length_min_in` decimal(7,2) DEFAULT NULL,
  `plant_length_max_in` decimal(7,2) DEFAULT NULL,
  `plant_width_min_in` decimal(7,2) DEFAULT NULL,
  `plant_width_max_in` decimal(7,2) DEFAULT NULL,
  `spacing_min_in` decimal(7,2) DEFAULT NULL,
  `spacing_max_in` decimal(7,2) DEFAULT NULL,
  `future_plant_id` int(11) DEFAULT NULL,
  `source_root` varchar(255) DEFAULT NULL,
  `source_url` varchar(500) DEFAULT NULL,
  `product_url` varchar(500) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_future_plant_id` (`future_plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=510 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `data_source_backup_20260929_063426`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `data_source_backup_20260929_063426` (
  `id` int(11) NOT NULL DEFAULT 0,
  `family` varchar(255) DEFAULT NULL,
  `picture` text DEFAULT NULL,
  `plant_name` varchar(255) DEFAULT NULL,
  `company` varchar(100) DEFAULT NULL,
  `local_store` varchar(255) DEFAULT NULL,
  `qty` int(11) DEFAULT 0,
  `died` int(11) DEFAULT 0,
  `new_total` int(11) DEFAULT 0,
  `year_planted` varchar(50) DEFAULT NULL,
  `split_amt` varchar(50) DEFAULT NULL,
  `sun_exposure` varchar(100) DEFAULT NULL,
  `cleanup_notes` text DEFAULT NULL,
  `care_notes` text DEFAULT NULL,
  `cuttings` varchar(255) DEFAULT NULL,
  `fertilizer` varchar(100) DEFAULT NULL,
  `size_spacing` varchar(100) DEFAULT NULL,
  `new_name` varchar(255) DEFAULT NULL,
  `flower_bed` varchar(100) DEFAULT NULL,
  `additional_pictures` text DEFAULT NULL,
  `maintenance_notes` text DEFAULT NULL,
  `attracts` text DEFAULT NULL,
  `zones` varchar(255) DEFAULT NULL,
  `water_needs` varchar(255) DEFAULT NULL,
  `deer_resistance` varchar(50) DEFAULT NULL,
  `season` varchar(100) DEFAULT NULL,
  `month_start` varchar(20) DEFAULT NULL,
  `month_end` varchar(20) DEFAULT NULL,
  `planting_month_start` varchar(20) DEFAULT NULL,
  `planting_month_end` varchar(20) DEFAULT NULL,
  `spacing` varchar(50) DEFAULT NULL,
  `admin_todo` varchar(255) DEFAULT NULL,
  `botanical_name` varchar(150) DEFAULT NULL,
  `soil_ph` varchar(255) DEFAULT NULL,
  `soil_type` varchar(255) DEFAULT NULL,
  `toxicity` varchar(255) DEFAULT NULL,
  `bloom_color` varchar(255) DEFAULT NULL,
  `foliage_color` varchar(255) DEFAULT NULL,
  `disease_susceptibility` text DEFAULT NULL,
  `propagation_method` varchar(255) DEFAULT NULL,
  `species` varchar(255) DEFAULT NULL,
  `size` varchar(255) DEFAULT NULL,
  `grow_months` varchar(255) DEFAULT NULL,
  `bloom_months` varchar(255) DEFAULT NULL,
  `average_cost` decimal(10,2) DEFAULT NULL,
  `extra_details` text DEFAULT NULL,
  `image_search_term` varchar(255) DEFAULT NULL,
  `plant_type` varchar(50) DEFAULT NULL,
  `image_verified` tinyint(1) DEFAULT 0,
  `stem_color` varchar(255) DEFAULT NULL,
  `genus` varchar(255) DEFAULT NULL,
  `leaf_shape` varchar(255) DEFAULT NULL,
  `leaf_type` varchar(255) DEFAULT NULL,
  `leaf_margin` varchar(255) DEFAULT NULL,
  `flower_symmetry` varchar(255) DEFAULT NULL,
  `flower_shape` varchar(255) DEFAULT NULL,
  `fruit_type` varchar(255) DEFAULT NULL,
  `fruit_shape` varchar(255) DEFAULT NULL,
  `fruit_color` varchar(255) DEFAULT NULL,
  `fruit_size` varchar(255) DEFAULT NULL,
  `fruit_edibility` varchar(255) DEFAULT NULL,
  `invasive_local` varchar(50) DEFAULT NULL,
  `native_status` varchar(50) DEFAULT NULL,
  `growth_rate` varchar(50) DEFAULT NULL,
  `planting_depth` varchar(255) DEFAULT NULL,
  `chilling_hours` varchar(255) DEFAULT NULL,
  `companion_plants` text DEFAULT NULL,
  `approved_jason` tinyint(1) DEFAULT 0,
  `approved_jun` tinyint(1) DEFAULT 0,
  `needs_review` tinyint(1) DEFAULT 0,
  `garden_beds` varchar(500) DEFAULT NULL,
  `aromatics` varchar(255) DEFAULT NULL,
  `flower_size` varchar(255) DEFAULT NULL,
  `is_evergreen` varchar(50) DEFAULT NULL,
  `quantity` int(11) DEFAULT 1,
  `year_added` varchar(255) DEFAULT NULL,
  `plant_length` varchar(255) DEFAULT NULL,
  `plant_width` varchar(255) DEFAULT NULL,
  `life_expectancy` varchar(255) DEFAULT NULL,
  `reproduction_type` varchar(255) DEFAULT NULL,
  `root_system` varchar(255) DEFAULT NULL,
  `root_morphology` varchar(255) DEFAULT NULL,
  `root_function` varchar(255) DEFAULT NULL,
  `grow_season` varchar(60) DEFAULT NULL,
  `died_year` smallint(5) unsigned DEFAULT NULL COMMENT 'Year the plant died (NULL = still alive)',
  `tag_done` tinyint(1) NOT NULL DEFAULT 0,
  `photo_position` varchar(30) DEFAULT '50% 50%',
  `photo_flip` tinyint(1) DEFAULT 0,
  `image_approved` tinyint(1) DEFAULT 0,
  `qr_code` mediumblob DEFAULT NULL,
  `barcode` varchar(20) DEFAULT NULL,
  `barcode_image` mediumblob DEFAULT NULL,
  `plant_length_min_in` decimal(7,2) DEFAULT NULL,
  `plant_length_max_in` decimal(7,2) DEFAULT NULL,
  `plant_width_min_in` decimal(7,2) DEFAULT NULL,
  `plant_width_max_in` decimal(7,2) DEFAULT NULL,
  `spacing_min_in` decimal(7,2) DEFAULT NULL,
  `spacing_max_in` decimal(7,2) DEFAULT NULL,
  `future_plant_id` int(11) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `data_source_srcbak_20260930_030827`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `data_source_srcbak_20260930_030827` (
  `id` int(11) NOT NULL DEFAULT 0,
  `family` varchar(255) DEFAULT NULL,
  `picture` text DEFAULT NULL,
  `plant_name` varchar(255) DEFAULT NULL,
  `company` varchar(100) DEFAULT NULL,
  `local_store` varchar(255) DEFAULT NULL,
  `qty` int(11) DEFAULT 0,
  `died` int(11) DEFAULT 0,
  `new_total` int(11) DEFAULT 0,
  `year_planted` varchar(50) DEFAULT NULL,
  `split_amt` varchar(50) DEFAULT NULL,
  `sun_exposure` varchar(100) DEFAULT NULL,
  `cleanup_notes` text DEFAULT NULL,
  `care_notes` text DEFAULT NULL,
  `cuttings` varchar(255) DEFAULT NULL,
  `fertilizer` varchar(100) DEFAULT NULL,
  `size_spacing` varchar(100) DEFAULT NULL,
  `new_name` varchar(255) DEFAULT NULL,
  `flower_bed` varchar(100) DEFAULT NULL,
  `additional_pictures` text DEFAULT NULL,
  `maintenance_notes` text DEFAULT NULL,
  `attracts` text DEFAULT NULL,
  `zones` varchar(255) DEFAULT NULL,
  `water_needs` varchar(255) DEFAULT NULL,
  `deer_resistance` varchar(50) DEFAULT NULL,
  `season` varchar(100) DEFAULT NULL,
  `month_start` varchar(20) DEFAULT NULL,
  `month_end` varchar(20) DEFAULT NULL,
  `planting_month_start` varchar(20) DEFAULT NULL,
  `planting_month_end` varchar(20) DEFAULT NULL,
  `spacing` varchar(50) DEFAULT NULL,
  `admin_todo` varchar(255) DEFAULT NULL,
  `botanical_name` varchar(150) DEFAULT NULL,
  `soil_ph` varchar(255) DEFAULT NULL,
  `soil_type` varchar(255) DEFAULT NULL,
  `toxicity` varchar(255) DEFAULT NULL,
  `bloom_color` varchar(255) DEFAULT NULL,
  `foliage_color` varchar(255) DEFAULT NULL,
  `disease_susceptibility` text DEFAULT NULL,
  `propagation_method` varchar(255) DEFAULT NULL,
  `species` varchar(255) DEFAULT NULL,
  `size` varchar(255) DEFAULT NULL,
  `grow_months` varchar(255) DEFAULT NULL,
  `bloom_months` varchar(255) DEFAULT NULL,
  `average_cost` decimal(10,2) DEFAULT NULL,
  `extra_details` text DEFAULT NULL,
  `image_search_term` varchar(255) DEFAULT NULL,
  `plant_type` varchar(50) DEFAULT NULL,
  `image_verified` tinyint(1) DEFAULT 0,
  `stem_color` varchar(255) DEFAULT NULL,
  `genus` varchar(255) DEFAULT NULL,
  `leaf_shape` varchar(255) DEFAULT NULL,
  `leaf_type` varchar(255) DEFAULT NULL,
  `leaf_margin` varchar(255) DEFAULT NULL,
  `flower_symmetry` varchar(255) DEFAULT NULL,
  `flower_shape` varchar(255) DEFAULT NULL,
  `fruit_type` varchar(255) DEFAULT NULL,
  `fruit_shape` varchar(255) DEFAULT NULL,
  `fruit_color` varchar(255) DEFAULT NULL,
  `fruit_size` varchar(255) DEFAULT NULL,
  `fruit_edibility` varchar(255) DEFAULT NULL,
  `invasive_local` varchar(50) DEFAULT NULL,
  `native_status` varchar(50) DEFAULT NULL,
  `growth_rate` varchar(50) DEFAULT NULL,
  `planting_depth` varchar(255) DEFAULT NULL,
  `chilling_hours` varchar(255) DEFAULT NULL,
  `companion_plants` text DEFAULT NULL,
  `approved_jason` tinyint(1) DEFAULT 0,
  `approved_jun` tinyint(1) DEFAULT 0,
  `needs_review` tinyint(1) DEFAULT 0,
  `garden_beds` varchar(500) DEFAULT NULL,
  `aromatics` varchar(255) DEFAULT NULL,
  `flower_size` varchar(255) DEFAULT NULL,
  `is_evergreen` varchar(50) DEFAULT NULL,
  `quantity` int(11) DEFAULT 1,
  `year_added` varchar(255) DEFAULT NULL,
  `plant_length` varchar(255) DEFAULT NULL,
  `plant_width` varchar(255) DEFAULT NULL,
  `life_expectancy` varchar(255) DEFAULT NULL,
  `reproduction_type` varchar(255) DEFAULT NULL,
  `root_system` varchar(255) DEFAULT NULL,
  `root_morphology` varchar(255) DEFAULT NULL,
  `root_function` varchar(255) DEFAULT NULL,
  `grow_season` varchar(60) DEFAULT NULL,
  `died_year` smallint(5) unsigned DEFAULT NULL COMMENT 'Year the plant died (NULL = still alive)',
  `tag_done` tinyint(1) NOT NULL DEFAULT 0,
  `photo_position` varchar(30) DEFAULT '50% 50%',
  `photo_flip` tinyint(1) DEFAULT 0,
  `image_approved` tinyint(1) DEFAULT 0,
  `qr_code` mediumblob DEFAULT NULL,
  `barcode` varchar(20) DEFAULT NULL,
  `barcode_image` mediumblob DEFAULT NULL,
  `plant_length_min_in` decimal(7,2) DEFAULT NULL,
  `plant_length_max_in` decimal(7,2) DEFAULT NULL,
  `plant_width_min_in` decimal(7,2) DEFAULT NULL,
  `plant_width_max_in` decimal(7,2) DEFAULT NULL,
  `spacing_min_in` decimal(7,2) DEFAULT NULL,
  `spacing_max_in` decimal(7,2) DEFAULT NULL,
  `future_plant_id` int(11) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `flower_bed_qr`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `flower_bed_qr` (
  `bed_name` varchar(120) NOT NULL,
  `qr_code` mediumblob NOT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`bed_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `future_plants`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `future_plants` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `family` varchar(255) DEFAULT NULL,
  `picture` text DEFAULT NULL,
  `plant_name` varchar(255) DEFAULT NULL,
  `company` varchar(100) DEFAULT NULL,
  `local_store` varchar(255) DEFAULT NULL,
  `qty` int(11) DEFAULT 0,
  `died` int(11) DEFAULT 0,
  `new_total` int(11) DEFAULT 0,
  `year_planted` varchar(50) DEFAULT NULL,
  `split_amt` varchar(50) DEFAULT NULL,
  `sun_exposure` varchar(100) DEFAULT NULL,
  `cleanup_notes` text DEFAULT NULL,
  `care_notes` text DEFAULT NULL,
  `cuttings` varchar(255) DEFAULT NULL,
  `fertilizer` varchar(100) DEFAULT NULL,
  `size_spacing` varchar(100) DEFAULT NULL,
  `new_name` varchar(255) DEFAULT NULL,
  `flower_bed` varchar(100) DEFAULT NULL,
  `additional_pictures` text DEFAULT NULL,
  `maintenance_notes` text DEFAULT NULL,
  `attracts` text DEFAULT NULL,
  `zones` varchar(255) DEFAULT NULL,
  `water_needs` varchar(255) DEFAULT NULL,
  `deer_resistance` varchar(50) DEFAULT NULL,
  `season` varchar(100) DEFAULT NULL,
  `month_start` varchar(20) DEFAULT NULL,
  `month_end` varchar(20) DEFAULT NULL,
  `planting_month_start` varchar(20) DEFAULT NULL,
  `planting_month_end` varchar(20) DEFAULT NULL,
  `spacing` varchar(50) DEFAULT NULL,
  `admin_todo` varchar(255) DEFAULT NULL,
  `botanical_name` varchar(150) DEFAULT NULL,
  `soil_ph` varchar(255) DEFAULT NULL,
  `soil_type` varchar(255) DEFAULT NULL,
  `toxicity` varchar(255) DEFAULT NULL,
  `bloom_color` varchar(255) DEFAULT NULL,
  `foliage_color` varchar(255) DEFAULT NULL,
  `disease_susceptibility` text DEFAULT NULL,
  `propagation_method` varchar(255) DEFAULT NULL,
  `species` varchar(255) DEFAULT NULL,
  `size` varchar(255) DEFAULT NULL,
  `grow_months` varchar(255) DEFAULT NULL,
  `bloom_months` varchar(255) DEFAULT NULL,
  `average_cost` decimal(10,2) DEFAULT NULL,
  `extra_details` text DEFAULT NULL,
  `image_search_term` varchar(255) DEFAULT NULL,
  `plant_type` varchar(50) DEFAULT NULL,
  `image_verified` tinyint(1) DEFAULT 0,
  `stem_color` varchar(255) DEFAULT NULL,
  `genus` varchar(255) DEFAULT NULL,
  `leaf_shape` varchar(255) DEFAULT NULL,
  `leaf_type` varchar(255) DEFAULT NULL,
  `leaf_margin` varchar(255) DEFAULT NULL,
  `flower_symmetry` varchar(255) DEFAULT NULL,
  `flower_shape` varchar(255) DEFAULT NULL,
  `fruit_type` varchar(255) DEFAULT NULL,
  `fruit_shape` varchar(255) DEFAULT NULL,
  `fruit_color` varchar(255) DEFAULT NULL,
  `fruit_size` varchar(255) DEFAULT NULL,
  `fruit_edibility` varchar(255) DEFAULT NULL,
  `invasive_local` varchar(50) DEFAULT NULL,
  `native_status` varchar(50) DEFAULT NULL,
  `growth_rate` varchar(50) DEFAULT NULL,
  `planting_depth` varchar(255) DEFAULT NULL,
  `chilling_hours` varchar(255) DEFAULT NULL,
  `companion_plants` text DEFAULT NULL,
  `approved_jason` tinyint(1) DEFAULT 0,
  `approved_jun` tinyint(1) DEFAULT 0,
  `needs_review` tinyint(1) DEFAULT 0,
  `garden_beds` varchar(500) DEFAULT NULL,
  `aromatics` varchar(255) DEFAULT NULL,
  `flower_size` varchar(255) DEFAULT NULL,
  `is_evergreen` varchar(50) DEFAULT NULL,
  `quantity` int(11) DEFAULT 1,
  `year_added` varchar(255) DEFAULT NULL,
  `plant_length` varchar(255) DEFAULT NULL,
  `plant_width` varchar(255) DEFAULT NULL,
  `life_expectancy` varchar(255) DEFAULT NULL,
  `reproduction_type` varchar(255) DEFAULT NULL,
  `root_system` varchar(255) DEFAULT NULL,
  `root_morphology` varchar(255) DEFAULT NULL,
  `root_function` varchar(255) DEFAULT NULL,
  `grow_season` varchar(60) DEFAULT NULL,
  `qr_code` mediumblob DEFAULT NULL,
  `barcode` varchar(20) DEFAULT NULL,
  `barcode_image` mediumblob DEFAULT NULL,
  `date_added` timestamp NULL DEFAULT current_timestamp(),
  `source_root` varchar(255) DEFAULT NULL,
  `source_url` varchar(500) DEFAULT NULL,
  `product_url` varchar(500) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_name_company` (`plant_name`,`company`)
) ENGINE=InnoDB AUTO_INCREMENT=8023 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `future_plants_srcbak_20260930_030826`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `future_plants_srcbak_20260930_030826` (
  `id` int(11) NOT NULL DEFAULT 0,
  `family` varchar(255) DEFAULT NULL,
  `picture` text DEFAULT NULL,
  `plant_name` varchar(255) DEFAULT NULL,
  `company` varchar(100) DEFAULT NULL,
  `local_store` varchar(255) DEFAULT NULL,
  `qty` int(11) DEFAULT 0,
  `died` int(11) DEFAULT 0,
  `new_total` int(11) DEFAULT 0,
  `year_planted` varchar(50) DEFAULT NULL,
  `split_amt` varchar(50) DEFAULT NULL,
  `sun_exposure` varchar(100) DEFAULT NULL,
  `cleanup_notes` text DEFAULT NULL,
  `care_notes` text DEFAULT NULL,
  `cuttings` varchar(255) DEFAULT NULL,
  `fertilizer` varchar(100) DEFAULT NULL,
  `size_spacing` varchar(100) DEFAULT NULL,
  `new_name` varchar(255) DEFAULT NULL,
  `flower_bed` varchar(100) DEFAULT NULL,
  `additional_pictures` text DEFAULT NULL,
  `maintenance_notes` text DEFAULT NULL,
  `attracts` text DEFAULT NULL,
  `zones` varchar(255) DEFAULT NULL,
  `water_needs` varchar(255) DEFAULT NULL,
  `deer_resistance` varchar(50) DEFAULT NULL,
  `season` varchar(100) DEFAULT NULL,
  `month_start` varchar(20) DEFAULT NULL,
  `month_end` varchar(20) DEFAULT NULL,
  `planting_month_start` varchar(20) DEFAULT NULL,
  `planting_month_end` varchar(20) DEFAULT NULL,
  `spacing` varchar(50) DEFAULT NULL,
  `admin_todo` varchar(255) DEFAULT NULL,
  `botanical_name` varchar(150) DEFAULT NULL,
  `soil_ph` varchar(255) DEFAULT NULL,
  `soil_type` varchar(255) DEFAULT NULL,
  `toxicity` varchar(255) DEFAULT NULL,
  `bloom_color` varchar(255) DEFAULT NULL,
  `foliage_color` varchar(255) DEFAULT NULL,
  `disease_susceptibility` text DEFAULT NULL,
  `propagation_method` varchar(255) DEFAULT NULL,
  `species` varchar(255) DEFAULT NULL,
  `size` varchar(255) DEFAULT NULL,
  `grow_months` varchar(255) DEFAULT NULL,
  `bloom_months` varchar(255) DEFAULT NULL,
  `average_cost` decimal(10,2) DEFAULT NULL,
  `extra_details` text DEFAULT NULL,
  `image_search_term` varchar(255) DEFAULT NULL,
  `plant_type` varchar(50) DEFAULT NULL,
  `image_verified` tinyint(1) DEFAULT 0,
  `stem_color` varchar(255) DEFAULT NULL,
  `genus` varchar(255) DEFAULT NULL,
  `leaf_shape` varchar(255) DEFAULT NULL,
  `leaf_type` varchar(255) DEFAULT NULL,
  `leaf_margin` varchar(255) DEFAULT NULL,
  `flower_symmetry` varchar(255) DEFAULT NULL,
  `flower_shape` varchar(255) DEFAULT NULL,
  `fruit_type` varchar(255) DEFAULT NULL,
  `fruit_shape` varchar(255) DEFAULT NULL,
  `fruit_color` varchar(255) DEFAULT NULL,
  `fruit_size` varchar(255) DEFAULT NULL,
  `fruit_edibility` varchar(255) DEFAULT NULL,
  `invasive_local` varchar(50) DEFAULT NULL,
  `native_status` varchar(50) DEFAULT NULL,
  `growth_rate` varchar(50) DEFAULT NULL,
  `planting_depth` varchar(255) DEFAULT NULL,
  `chilling_hours` varchar(255) DEFAULT NULL,
  `companion_plants` text DEFAULT NULL,
  `approved_jason` tinyint(1) DEFAULT 0,
  `approved_jun` tinyint(1) DEFAULT 0,
  `needs_review` tinyint(1) DEFAULT 0,
  `garden_beds` varchar(500) DEFAULT NULL,
  `aromatics` varchar(255) DEFAULT NULL,
  `flower_size` varchar(255) DEFAULT NULL,
  `is_evergreen` varchar(50) DEFAULT NULL,
  `quantity` int(11) DEFAULT 1,
  `year_added` varchar(255) DEFAULT NULL,
  `plant_length` varchar(255) DEFAULT NULL,
  `plant_width` varchar(255) DEFAULT NULL,
  `life_expectancy` varchar(255) DEFAULT NULL,
  `reproduction_type` varchar(255) DEFAULT NULL,
  `root_system` varchar(255) DEFAULT NULL,
  `root_morphology` varchar(255) DEFAULT NULL,
  `root_function` varchar(255) DEFAULT NULL,
  `grow_season` varchar(60) DEFAULT NULL,
  `qr_code` mediumblob DEFAULT NULL,
  `barcode` varchar(20) DEFAULT NULL,
  `barcode_image` mediumblob DEFAULT NULL,
  `date_added` timestamp NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `garden_bed`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `garden_bed` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `name` varchar(120) NOT NULL,
  `shape` text NOT NULL,
  `sun` varchar(60) DEFAULT '',
  `soil` varchar(120) DEFAULT '',
  `notes` varchar(500) DEFAULT '',
  `created_by` varchar(190) DEFAULT '',
  `created_at` timestamp NULL DEFAULT current_timestamp(),
  `updated_at` timestamp NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `photo_id` int(11) DEFAULT NULL,
  `rough` tinyint(4) NOT NULL DEFAULT 0,
  `traced_scale` decimal(12,6) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_bed_name` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `garden_blog`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `garden_blog` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `author` varchar(50) DEFAULT 'Jun',
  `title` varchar(255) NOT NULL,
  `content` text NOT NULL,
  `picture` text DEFAULT NULL,
  `created_at` datetime DEFAULT current_timestamp(),
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `garden_object`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `garden_object` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `bed_id` int(11) DEFAULT NULL,
  `kind` varchar(20) NOT NULL,
  `label` varchar(160) DEFAULT '',
  `x` decimal(9,2) NOT NULL,
  `y` decimal(9,2) NOT NULL,
  `w` decimal(9,2) NOT NULL,
  `d` decimal(9,2) NOT NULL,
  `h` decimal(9,2) NOT NULL DEFAULT 0.00,
  `rotation` decimal(6,2) NOT NULL DEFAULT 0.00,
  `colour` varchar(20) DEFAULT '',
  `photo_id` int(11) DEFAULT NULL,
  `shape` text DEFAULT NULL,
  `months` char(12) NOT NULL DEFAULT 'JFMAMJJASOND',
  `glows` tinyint(4) NOT NULL DEFAULT 0,
  `lies_flat` tinyint(4) NOT NULL DEFAULT 0,
  `notes` varchar(400) DEFAULT '',
  `placed_by` varchar(190) DEFAULT '',
  `placed_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `k_bed` (`bed_id`),
  KEY `k_kind` (`kind`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `garden_photo`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `garden_photo` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `bed_id` int(11) DEFAULT NULL,
  `kind` varchar(20) NOT NULL DEFAULT 'overhead',
  `filename` varchar(120) NOT NULL,
  `width` int(11) NOT NULL,
  `height` int(11) NOT NULL,
  `in_per_px` decimal(12,6) DEFAULT NULL,
  `cal_note` varchar(200) DEFAULT '',
  `label` varchar(160) DEFAULT '',
  `uploaded_by` varchar(190) DEFAULT '',
  `uploaded_at` timestamp NULL DEFAULT current_timestamp(),
  `stood_x` decimal(9,2) DEFAULT NULL,
  `stood_y` decimal(9,2) DEFAULT NULL,
  `heading` decimal(6,2) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `k_bed` (`bed_id`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `garden_planting`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `garden_planting` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `bed_id` int(11) NOT NULL,
  `plant_id` int(11) NOT NULL,
  `x` decimal(9,2) NOT NULL,
  `y` decimal(9,2) NOT NULL,
  `qty` int(11) NOT NULL DEFAULT 1,
  `placed_by` varchar(190) DEFAULT '',
  `placed_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `k_bed` (`bed_id`),
  KEY `k_plant` (`plant_id`),
  CONSTRAINT `fk_planting_bed` FOREIGN KEY (`bed_id`) REFERENCES `garden_bed` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `login_attempts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `login_attempts` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `ip` varchar(45) NOT NULL,
  `attempted_at` datetime NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `idx_ip_time` (`ip`,`attempted_at`)
) ENGINE=InnoDB AUTO_INCREMENT=14 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_barcodes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_barcodes` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `source_table` enum('data_source','future_plants') NOT NULL DEFAULT 'data_source',
  `size_label` varchar(60) DEFAULT NULL,
  `barcode` varchar(20) NOT NULL,
  `barcode_image` mediumblob DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_barcode` (`barcode`),
  KEY `idx_plant` (`plant_id`,`source_table`),
  KEY `idx_plant_size` (`plant_id`,`source_table`,`size_label`)
) ENGINE=InnoDB AUTO_INCREMENT=343 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_checkup`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_checkup` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `looked_at` text NOT NULL,
  `consistent` text NOT NULL,
  `confirm` text NOT NULL,
  `raw` mediumtext DEFAULT NULL,
  `photo` mediumtext DEFAULT NULL,
  `asked_by` varchar(190) DEFAULT '',
  `asked_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `k_plant` (`plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_completions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_completions` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `month_index` int(11) NOT NULL,
  `year` int(11) NOT NULL,
  `completed_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_task` (`plant_id`,`month_index`,`year`),
  CONSTRAINT `pc_fk_plant` FOREIGN KEY (`plant_id`) REFERENCES `data_source` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_gallery`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_gallery` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `filename` varchar(255) NOT NULL,
  `likes` int(11) DEFAULT 0,
  `notes` text DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_pg_plant` (`plant_id`),
  CONSTRAINT `pg_fk_plant` FOREIGN KEY (`plant_id`) REFERENCES `data_source` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_purchase`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_purchase` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) DEFAULT NULL,
  `line_text` varchar(300) NOT NULL,
  `matched_name` varchar(220) DEFAULT '',
  `quantity` int(11) NOT NULL DEFAULT 1,
  `unit_price` decimal(10,2) DEFAULT NULL,
  `line_total` decimal(10,2) DEFAULT NULL,
  `store` varchar(200) DEFAULT '',
  `bought_on` date DEFAULT NULL,
  `receipt_ref` varchar(200) DEFAULT '',
  `paperless_id` int(11) DEFAULT NULL,
  `confidence` decimal(4,3) DEFAULT NULL,
  `added_by` varchar(190) DEFAULT '',
  `added_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `k_plant` (`plant_id`),
  KEY `k_when` (`bought_on`),
  KEY `k_receipt` (`receipt_ref`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_quantity_log`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_quantity_log` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `event_type` enum('purchased','died','removed','other') NOT NULL,
  `quantity` smallint(6) NOT NULL COMMENT 'Positive = added, negative = lost/removed',
  `event_date` date NOT NULL COMMENT 'Monday of the logged week (allows backdating)',
  `event_week` tinyint(3) unsigned NOT NULL COMMENT 'ISO week number 1-53',
  `event_month` tinyint(3) unsigned NOT NULL COMMENT '1-12, derived from event_date',
  `event_year` smallint(5) unsigned NOT NULL COMMENT 'Derived from event_date',
  `notes` varchar(500) DEFAULT NULL,
  `recorded_by` varchar(100) DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `idx_plant` (`plant_id`),
  KEY `idx_period` (`event_date`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_reviews`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_reviews` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `rating` tinyint(4) NOT NULL,
  `comment` text DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `idx_pr_plant` (`plant_id`),
  CONSTRAINT `pr_fk_plant` FOREIGN KEY (`plant_id`) REFERENCES `data_source` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `plant_tag_photo`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `plant_tag_photo` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) DEFAULT NULL,
  `side` varchar(10) NOT NULL DEFAULT 'front',
  `filename` varchar(200) NOT NULL,
  `width` int(11) NOT NULL,
  `height` int(11) NOT NULL,
  `taken_by` varchar(190) DEFAULT '',
  `taken_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `k_plant` (`plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `soil_type_backup_20260928_043124`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `soil_type_backup_20260928_043124` (
  `id` int(11) NOT NULL,
  `old_value` text DEFAULT NULL,
  `new_value` text DEFAULT NULL,
  `taken_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_done`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_done` (
  `plant_id` int(11) NOT NULL,
  `username` varchar(50) NOT NULL,
  `done_at` timestamp NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`plant_id`,`username`),
  KEY `idx_user` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_favorites`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_favorites` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `username` varchar(50) NOT NULL,
  `plant_id` int(11) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_user_plant` (`username`,`plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=131 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_likes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_likes` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `username` varchar(255) NOT NULL,
  `plant_id` int(11) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_like` (`username`,`plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=31 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_notes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_notes` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) NOT NULL,
  `username` varchar(50) NOT NULL,
  `note_html` mediumtext NOT NULL DEFAULT '',
  `created_at` datetime NOT NULL DEFAULT current_timestamp(),
  `updated_at` datetime NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_plant_user` (`plant_id`,`username`),
  KEY `idx_username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_prefs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_prefs` (
  `username` varchar(100) NOT NULL,
  `pref_key` varchar(50) NOT NULL,
  `pref_value` varchar(200) NOT NULL,
  `updated_at` timestamp NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  PRIMARY KEY (`username`,`pref_key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `users` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `username` varchar(50) NOT NULL,
  `password` varchar(255) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `video_source`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `video_source` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `source_id` varchar(64) NOT NULL,
  `url` varchar(500) NOT NULL,
  `title` varchar(300) DEFAULT '',
  `channel` varchar(200) DEFAULT '',
  `duration_s` int(11) DEFAULT NULL,
  `published` varchar(20) DEFAULT '',
  `transcript` mediumtext DEFAULT NULL,
  `video_path` varchar(500) DEFAULT '',
  `keep_video` tinyint(4) NOT NULL DEFAULT 0,
  `status` varchar(16) NOT NULL DEFAULT 'queued',
  `note` varchar(600) DEFAULT '',
  `tips_found` int(11) NOT NULL DEFAULT 0,
  `added_by` varchar(190) DEFAULT '',
  `added_at` timestamp NULL DEFAULT current_timestamp(),
  `done_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_source` (`source_id`),
  KEY `k_status` (`status`,`added_at`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `vocab_alias`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `vocab_alias` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `field_name` varchar(64) NOT NULL,
  `raw_key` varchar(191) NOT NULL,
  `canonical` varchar(191) NOT NULL,
  `approved_by` varchar(190) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_field_raw` (`field_name`,`raw_key`),
  KEY `idx_field` (`field_name`)
) ENGINE=InnoDB AUTO_INCREMENT=12 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `vocab_proposal`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `vocab_proposal` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) DEFAULT NULL,
  `field_name` varchar(64) NOT NULL,
  `raw_value` varchar(500) NOT NULL,
  `suggested` varchar(191) DEFAULT NULL,
  `previous_value` varchar(500) DEFAULT NULL,
  `confidence` float NOT NULL DEFAULT 0,
  `how` varchar(16) NOT NULL,
  `source` varchar(500) DEFAULT NULL,
  `status` enum('pending','approved','rejected','confirmed','reverted') NOT NULL DEFAULT 'pending',
  `decided_by` varchar(190) DEFAULT NULL,
  `decided_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `reviewed_by` varchar(190) DEFAULT NULL,
  `reviewed_at` timestamp NULL DEFAULT NULL,
  `review_note` varchar(500) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_status` (`status`),
  KEY `idx_plant` (`plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `weather`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `weather` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `obs_time` datetime NOT NULL,
  `location` varchar(64) NOT NULL DEFAULT 'clifton_park',
  `source` varchar(32) NOT NULL DEFAULT 'live',
  `temperature` float DEFAULT NULL,
  `feels_like` float DEFAULT NULL,
  `humidity` float DEFAULT NULL,
  `dew_point` float DEFAULT NULL,
  `precipitation` float DEFAULT NULL,
  `rain` float DEFAULT NULL,
  `snowfall` float DEFAULT NULL,
  `wind_speed` float DEFAULT NULL,
  `wind_direction` float DEFAULT NULL,
  `wind_gusts` float DEFAULT NULL,
  `cloud_cover` float DEFAULT NULL,
  `pressure` float DEFAULT NULL,
  `uv_index` float DEFAULT NULL,
  `visibility` float DEFAULT NULL,
  `soil_temp_6in` float DEFAULT NULL,
  `soil_moisture_6in` float DEFAULT NULL,
  `noaa_temp_max` float DEFAULT NULL,
  `noaa_temp_min` float DEFAULT NULL,
  `noaa_temp_avg` float DEFAULT NULL,
  `noaa_precip` float DEFAULT NULL,
  `noaa_snowfall` float DEFAULT NULL,
  `noaa_snow_depth` float DEFAULT NULL,
  `noaa_wind_avg` float DEFAULT NULL,
  `noaa_wind_gust` float DEFAULT NULL,
  `noaa_wind_peak` float DEFAULT NULL,
  `noaa_humidity_avg` float DEFAULT NULL,
  `noaa_humidity_min` float DEFAULT NULL,
  `noaa_humidity_max` float DEFAULT NULL,
  `noaa_dew_point` float DEFAULT NULL,
  `noaa_pressure` float DEFAULT NULL,
  `noaa_sunshine_min` float DEFAULT NULL,
  `noaa_sunshine_pct` float DEFAULT NULL,
  `noaa_events` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_obs` (`obs_time`,`source`,`location`),
  KEY `idx_obs_time` (`obs_time`),
  KEY `idx_src_time` (`source`,`obs_time`)
) ENGINE=InnoDB AUTO_INCREMENT=60469 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `weather_forecast`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `weather_forecast` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `obs_time` datetime NOT NULL,
  `temperature` float DEFAULT NULL,
  `generated_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_fc` (`obs_time`),
  KEY `idx_fc_time` (`obs_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `wiki_draft`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8mb4 */;
CREATE TABLE `wiki_draft` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `plant_id` int(11) DEFAULT NULL,
  `kind` varchar(16) NOT NULL,
  `destination` varchar(16) NOT NULL DEFAULT 'garden',
  `page_path` varchar(220) NOT NULL,
  `title` varchar(220) NOT NULL,
  `section` varchar(120) NOT NULL DEFAULT '',
  `proposed` mediumtext NOT NULL,
  `existing` mediumtext DEFAULT NULL,
  `source_url` varchar(500) DEFAULT '',
  `source_name` varchar(220) DEFAULT '',
  `images` mediumtext DEFAULT NULL,
  `status` varchar(16) NOT NULL DEFAULT 'pending',
  `decided_by` varchar(190) DEFAULT '',
  `decided_at` timestamp NULL DEFAULT NULL,
  `result` varchar(600) DEFAULT '',
  `created_at` timestamp NULL DEFAULT current_timestamp(),
  PRIMARY KEY (`id`),
  KEY `k_status` (`status`,`created_at`),
  KEY `k_plant` (`plant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=42 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

USE `juns_garden_db`;
/*!50001 DROP VIEW IF EXISTS `Annuals`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_uca1400_ai_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`dbadmin`@`%` SQL SECURITY DEFINER */
/*!50001 VIEW `Annuals` AS select `data_source`.`id` AS `id`,`data_source`.`family` AS `family`,`data_source`.`picture` AS `picture`,`data_source`.`plant_name` AS `plant_name`,`data_source`.`company` AS `company`,`data_source`.`local_store` AS `local_store`,`data_source`.`qty` AS `qty`,`data_source`.`died` AS `died`,`data_source`.`new_total` AS `new_total`,`data_source`.`year_planted` AS `year_planted`,`data_source`.`split_amt` AS `split_amt`,`data_source`.`sun_exposure` AS `sun_exposure`,`data_source`.`cleanup_notes` AS `cleanup_notes`,`data_source`.`care_notes` AS `care_notes`,`data_source`.`cuttings` AS `cuttings`,`data_source`.`fertilizer` AS `fertilizer`,`data_source`.`size_spacing` AS `size_spacing`,`data_source`.`new_name` AS `new_name`,`data_source`.`flower_bed` AS `flower_bed`,`data_source`.`additional_pictures` AS `additional_pictures`,`data_source`.`maintenance_notes` AS `maintenance_notes`,`data_source`.`attracts` AS `attracts`,`data_source`.`zones` AS `zones`,`data_source`.`water_needs` AS `water_needs`,`data_source`.`deer_resistance` AS `deer_resistance`,`data_source`.`season` AS `season`,`data_source`.`month_start` AS `month_start`,`data_source`.`month_end` AS `month_end`,`data_source`.`planting_month_start` AS `planting_month_start`,`data_source`.`planting_month_end` AS `planting_month_end`,`data_source`.`spacing` AS `spacing`,`data_source`.`admin_todo` AS `admin_todo`,`data_source`.`botanical_name` AS `botanical_name`,`data_source`.`soil_ph` AS `soil_ph`,`data_source`.`soil_type` AS `soil_type`,`data_source`.`toxicity` AS `toxicity`,`data_source`.`bloom_color` AS `bloom_color`,`data_source`.`foliage_color` AS `foliage_color`,`data_source`.`disease_susceptibility` AS `disease_susceptibility`,`data_source`.`propagation_method` AS `propagation_method`,`data_source`.`species` AS `species`,`data_source`.`size` AS `size`,`data_source`.`grow_months` AS `grow_months`,`data_source`.`bloom_months` AS `bloom_months`,`data_source`.`average_cost` AS `average_cost`,`data_source`.`extra_details` AS `extra_details`,`data_source`.`image_search_term` AS `image_search_term`,`data_source`.`plant_type` AS `plant_type`,`data_source`.`image_verified` AS `image_verified`,`data_source`.`stem_color` AS `stem_color`,`data_source`.`genus` AS `genus`,`data_source`.`leaf_shape` AS `leaf_shape`,`data_source`.`leaf_type` AS `leaf_type`,`data_source`.`leaf_margin` AS `leaf_margin`,`data_source`.`flower_symmetry` AS `flower_symmetry`,`data_source`.`flower_shape` AS `flower_shape`,`data_source`.`fruit_type` AS `fruit_type`,`data_source`.`fruit_shape` AS `fruit_shape`,`data_source`.`fruit_color` AS `fruit_color`,`data_source`.`fruit_size` AS `fruit_size`,`data_source`.`fruit_edibility` AS `fruit_edibility`,`data_source`.`invasive_local` AS `invasive_local`,`data_source`.`native_status` AS `native_status`,`data_source`.`growth_rate` AS `growth_rate`,`data_source`.`planting_depth` AS `planting_depth`,`data_source`.`chilling_hours` AS `chilling_hours`,`data_source`.`companion_plants` AS `companion_plants`,`data_source`.`approved_jason` AS `approved_jason`,`data_source`.`approved_jun` AS `approved_jun`,`data_source`.`needs_review` AS `needs_review`,`data_source`.`garden_beds` AS `garden_beds`,`data_source`.`aromatics` AS `aromatics`,`data_source`.`flower_size` AS `flower_size`,`data_source`.`is_evergreen` AS `is_evergreen`,`data_source`.`quantity` AS `quantity`,`data_source`.`year_added` AS `year_added`,`data_source`.`plant_length` AS `plant_length`,`data_source`.`plant_width` AS `plant_width`,`data_source`.`life_expectancy` AS `life_expectancy`,`data_source`.`reproduction_type` AS `reproduction_type`,`data_source`.`root_system` AS `root_system`,`data_source`.`root_morphology` AS `root_morphology`,`data_source`.`root_function` AS `root_function`,`data_source`.`grow_season` AS `grow_season`,`data_source`.`died_year` AS `died_year`,`data_source`.`tag_done` AS `tag_done`,`data_source`.`photo_position` AS `photo_position`,`data_source`.`photo_flip` AS `photo_flip`,`data_source`.`image_approved` AS `image_approved`,`data_source`.`qr_code` AS `qr_code`,`data_source`.`barcode` AS `barcode`,`data_source`.`barcode_image` AS `barcode_image`,`data_source`.`plant_length_min_in` AS `plant_length_min_in`,`data_source`.`plant_length_max_in` AS `plant_length_max_in`,`data_source`.`plant_width_min_in` AS `plant_width_min_in`,`data_source`.`plant_width_max_in` AS `plant_width_max_in`,`data_source`.`spacing_min_in` AS `spacing_min_in`,`data_source`.`spacing_max_in` AS `spacing_max_in` from `data_source` where `data_source`.`plant_type` = 'Annual' */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;
/*!50001 DROP VIEW IF EXISTS `Perennials`*/;
/*!50001 SET @saved_cs_client          = @@character_set_client */;
/*!50001 SET @saved_cs_results         = @@character_set_results */;
/*!50001 SET @saved_col_connection     = @@collation_connection */;
/*!50001 SET character_set_client      = utf8mb4 */;
/*!50001 SET character_set_results     = utf8mb4 */;
/*!50001 SET collation_connection      = utf8mb4_uca1400_ai_ci */;
/*!50001 CREATE ALGORITHM=UNDEFINED */
/*!50013 DEFINER=`dbadmin`@`%` SQL SECURITY DEFINER */
/*!50001 VIEW `Perennials` AS select `data_source`.`id` AS `id`,`data_source`.`family` AS `family`,`data_source`.`picture` AS `picture`,`data_source`.`plant_name` AS `plant_name`,`data_source`.`company` AS `company`,`data_source`.`local_store` AS `local_store`,`data_source`.`qty` AS `qty`,`data_source`.`died` AS `died`,`data_source`.`new_total` AS `new_total`,`data_source`.`year_planted` AS `year_planted`,`data_source`.`split_amt` AS `split_amt`,`data_source`.`sun_exposure` AS `sun_exposure`,`data_source`.`cleanup_notes` AS `cleanup_notes`,`data_source`.`care_notes` AS `care_notes`,`data_source`.`cuttings` AS `cuttings`,`data_source`.`fertilizer` AS `fertilizer`,`data_source`.`size_spacing` AS `size_spacing`,`data_source`.`new_name` AS `new_name`,`data_source`.`flower_bed` AS `flower_bed`,`data_source`.`additional_pictures` AS `additional_pictures`,`data_source`.`maintenance_notes` AS `maintenance_notes`,`data_source`.`attracts` AS `attracts`,`data_source`.`zones` AS `zones`,`data_source`.`water_needs` AS `water_needs`,`data_source`.`deer_resistance` AS `deer_resistance`,`data_source`.`season` AS `season`,`data_source`.`month_start` AS `month_start`,`data_source`.`month_end` AS `month_end`,`data_source`.`planting_month_start` AS `planting_month_start`,`data_source`.`planting_month_end` AS `planting_month_end`,`data_source`.`spacing` AS `spacing`,`data_source`.`admin_todo` AS `admin_todo`,`data_source`.`botanical_name` AS `botanical_name`,`data_source`.`soil_ph` AS `soil_ph`,`data_source`.`soil_type` AS `soil_type`,`data_source`.`toxicity` AS `toxicity`,`data_source`.`bloom_color` AS `bloom_color`,`data_source`.`foliage_color` AS `foliage_color`,`data_source`.`disease_susceptibility` AS `disease_susceptibility`,`data_source`.`propagation_method` AS `propagation_method`,`data_source`.`species` AS `species`,`data_source`.`size` AS `size`,`data_source`.`grow_months` AS `grow_months`,`data_source`.`bloom_months` AS `bloom_months`,`data_source`.`average_cost` AS `average_cost`,`data_source`.`extra_details` AS `extra_details`,`data_source`.`image_search_term` AS `image_search_term`,`data_source`.`plant_type` AS `plant_type`,`data_source`.`image_verified` AS `image_verified`,`data_source`.`stem_color` AS `stem_color`,`data_source`.`genus` AS `genus`,`data_source`.`leaf_shape` AS `leaf_shape`,`data_source`.`leaf_type` AS `leaf_type`,`data_source`.`leaf_margin` AS `leaf_margin`,`data_source`.`flower_symmetry` AS `flower_symmetry`,`data_source`.`flower_shape` AS `flower_shape`,`data_source`.`fruit_type` AS `fruit_type`,`data_source`.`fruit_shape` AS `fruit_shape`,`data_source`.`fruit_color` AS `fruit_color`,`data_source`.`fruit_size` AS `fruit_size`,`data_source`.`fruit_edibility` AS `fruit_edibility`,`data_source`.`invasive_local` AS `invasive_local`,`data_source`.`native_status` AS `native_status`,`data_source`.`growth_rate` AS `growth_rate`,`data_source`.`planting_depth` AS `planting_depth`,`data_source`.`chilling_hours` AS `chilling_hours`,`data_source`.`companion_plants` AS `companion_plants`,`data_source`.`approved_jason` AS `approved_jason`,`data_source`.`approved_jun` AS `approved_jun`,`data_source`.`needs_review` AS `needs_review`,`data_source`.`garden_beds` AS `garden_beds`,`data_source`.`aromatics` AS `aromatics`,`data_source`.`flower_size` AS `flower_size`,`data_source`.`is_evergreen` AS `is_evergreen`,`data_source`.`quantity` AS `quantity`,`data_source`.`year_added` AS `year_added`,`data_source`.`plant_length` AS `plant_length`,`data_source`.`plant_width` AS `plant_width`,`data_source`.`life_expectancy` AS `life_expectancy`,`data_source`.`reproduction_type` AS `reproduction_type`,`data_source`.`root_system` AS `root_system`,`data_source`.`root_morphology` AS `root_morphology`,`data_source`.`root_function` AS `root_function`,`data_source`.`grow_season` AS `grow_season`,`data_source`.`died_year` AS `died_year`,`data_source`.`tag_done` AS `tag_done`,`data_source`.`photo_position` AS `photo_position`,`data_source`.`photo_flip` AS `photo_flip`,`data_source`.`image_approved` AS `image_approved`,`data_source`.`qr_code` AS `qr_code`,`data_source`.`barcode` AS `barcode`,`data_source`.`barcode_image` AS `barcode_image`,`data_source`.`plant_length_min_in` AS `plant_length_min_in`,`data_source`.`plant_length_max_in` AS `plant_length_max_in`,`data_source`.`plant_width_min_in` AS `plant_width_min_in`,`data_source`.`plant_width_max_in` AS `plant_width_max_in`,`data_source`.`spacing_min_in` AS `spacing_min_in`,`data_source`.`spacing_max_in` AS `spacing_max_in` from `data_source` where `data_source`.`plant_type` = 'Perennial' */;
/*!50001 SET character_set_client      = @saved_cs_client */;
/*!50001 SET character_set_results     = @saved_cs_results */;
/*!50001 SET collation_connection      = @saved_col_connection */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*M!100616 SET NOTE_VERBOSITY=@OLD_NOTE_VERBOSITY */;

