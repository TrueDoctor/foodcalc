module MealModal exposing (..)

import SearchList



-- TYPES


type Meal
    = Meal Internals
    | NewMeal


type alias Internals =
    { event_id : Int
    , recipe_id : Int
    , recipe_name : String
    , comment : Maybe String
    , place_id : Int
    , place_name : String
    , start_time : String
    , end_time : String
    , weight : Float
    , energy : Float
    , price : String
    , servings : Int
    }



-- MESSAGES


type MealMsg
    = MealSearchMsg (SearchList.SearchListMsg Meal)
    | MealName String
    | MealComment String
    | MealPlace String
    | MealStartTime String
    | MealEndTime String
    | MealWeight String
    | MealEnergy String
    | MealPrice String
    | MealServings String
    | AddNewMeal



-- BUILDERS
-- Getters

