# Branchrestriction

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**links** | Option<[**serde_json::Value**](.md)> |  | [optional]
**id** | Option<**i32**> | The branch restriction status' id. | [optional]
**kind** | **String** | The type of restriction that is being applied. | 
**branch_match_kind** | **String** | Indicates how the restriction is matched against a branch. The default is `glob`. | 
**branch_type** | Option<**String**> | Apply the restriction to branches of this type. Active when `branch_match_kind` is `branching_model`. The branch type will be calculated using the branching model configured for the repository. | [optional]
**pattern** | **String** | Apply the restriction to branches that match this pattern. Active when `branch_match_kind` is `glob`. Will be empty when `branch_match_kind` is `branching_model`. | 
**value** | Option<**i32**> | Value with kind-specific semantics:  * `require_approvals_to_merge` uses it to require a minimum number of approvals on a PR.  * `require_default_reviewer_approvals_to_merge` uses it to require a minimum number of approvals from default reviewers on a PR.  * `require_passing_builds_to_merge` uses it to require a minimum number of passing builds.  * `require_commits_behind` uses it to require the current branch is up to a maximum number of commits behind it destination. | [optional]
**users** | Option<[**Vec<models::Account>**](account.md)> |  | [optional]
**groups** | Option<[**Vec<models::Group>**](group.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


